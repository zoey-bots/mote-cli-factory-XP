use std::io::{self, Write};

const USAGE: &str = "Usage: mote [--jsonl] <manifest.yaml> <task>\n       mote --version\n";
const VERSION_LINE: &str = "mote 0.13.1\n";

const MANIFEST_FIELDS: &[&str] = &[
    "name",
    "workspace",
    "capabilities",
    "max_iterations",
    "max_per_tool",
    "max_runtime_seconds",
    "command_timeout_seconds",
    "max_output_bytes",
    "shell",
    "output_file",
    "models",
    "decision",
];

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = run(
        &args,
        |path| std::fs::read_to_string(path),
        &mut io::stdout(),
        &mut io::stderr(),
    );
    std::process::exit(code);
}

fn run(
    args: &[String],
    read: impl Fn(&str) -> io::Result<String>,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> i32 {
    let mut jsonl = false;
    let mut positionals: Vec<String> = Vec::new();

    for arg in args {
        if arg == "--help" || arg == "-h" {
            let _ = out.write_all(USAGE.as_bytes());
            return 0;
        }
        if arg == "--version" {
            let _ = out.write_all(VERSION_LINE.as_bytes());
            return 0;
        }
        if arg == "--jsonl" {
            jsonl = true;
            continue;
        }
        positionals.push(arg.clone());
    }

    if positionals.len() < 2 {
        let _ = err.write_all(USAGE.as_bytes());
        return 2;
    }

    let manifest_path = &positionals[0];
    let content = match read(manifest_path) {
        Ok(c) => c,
        Err(e) => {
            let _ = write!(err, "unable to read manifest: {}\n", e);
            return 2;
        }
    };

    if let Err(msg) = validate_manifest(&content) {
        let _ = write!(err, "invalid manifest: {}\n", msg);
        return 2;
    }

    let _ = write!(
        err,
        "mote: manifest accepted; agent runtime and tool execution are not implemented in this offline reconstruction{}\n",
        if jsonl {
            " (jsonl output unavailable)"
        } else {
            ""
        }
    );
    1
}

enum Scalar {
    Null,
    Bool,
    Integer(String),
    Float(String),
    Str,
}

fn classify_scalar(raw: &str) -> Scalar {
    let t = raw.trim();
    if t.is_empty() || t == "null" || t == "Null" || t == "NULL" || t == "~" {
        return Scalar::Null;
    }
    if t == "true" || t == "false" {
        return Scalar::Bool;
    }
    if t.len() >= 2 {
        let mut chars = t.chars();
        let first = chars.next().unwrap_or(' ');
        let last = chars.next_back().unwrap_or(' ');
        if (first == '"' && last == '"') || (first == '\'' && last == '\'') {
            return Scalar::Str;
        }
    }
    if t.parse::<i64>().is_ok() {
        return Scalar::Integer(t.to_string());
    }
    if (t.contains('.') || t.contains('e') || t.contains('E')) && t.parse::<f64>().is_ok() {
        return Scalar::Float(t.to_string());
    }
    Scalar::Str
}

fn strip_quotes(s: &str) -> String {
    let t = s.trim();
    if t.len() >= 2 {
        let first = t.chars().next().unwrap_or(' ');
        let last = t.chars().next_back().unwrap_or(' ');
        if (first == '"' && last == '"') || (first == '\'' && last == '\'') {
            return t[1..t.len() - 1].to_string();
        }
    }
    t.to_string()
}

fn split_key_value(line: &str) -> Option<(String, String)> {
    let chars: Vec<char> = line.chars().collect();
    let mut in_single = false;
    let mut in_double = false;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\'' && !in_double {
            in_single = !in_single;
        } else if c == '"' && !in_single {
            in_double = !in_double;
        } else if c == ':' && !in_single && !in_double {
            let next = chars.get(i + 1).copied();
            if next.is_none() || next == Some(' ') || next == Some('\t') {
                let key_raw: String = chars[..i].iter().collect();
                let key = key_raw.trim();
                if key.is_empty() {
                    return None;
                }
                let value: String = chars[i + 1..].iter().collect();
                return Some((strip_quotes(key), value.trim().to_string()));
            }
        }
        i += 1;
    }
    None
}

fn parse_flow_map(first: &str) -> Result<Vec<(String, String)>, String> {
    let t = first.trim();
    let inner = t.strip_prefix('{').unwrap_or(t);
    let inner = inner.strip_suffix('}').unwrap_or(inner);
    let mut items: Vec<(String, String)> = Vec::new();
    if inner.trim().is_empty() {
        return Ok(items);
    }
    let mut depth: i32 = 0;
    let mut cur = String::new();
    let push_item = |cur: &str, items: &mut Vec<(String, String)>| -> Result<(), String> {
        if cur.trim().is_empty() {
            return Ok(());
        }
        match split_key_value(cur) {
            Some(kv) => {
                items.push(kv);
                Ok(())
            }
            None => Err("did not find expected ':' while scanning a simple key".to_string()),
        }
    };
    for c in inner.chars() {
        match c {
            '[' | '{' => {
                depth += 1;
                cur.push(c);
            }
            ']' | '}' => {
                depth -= 1;
                cur.push(c);
            }
            ',' if depth == 0 => {
                push_item(&cur, &mut items)?;
                cur = String::new();
            }
            _ => cur.push(c),
        }
    }
    push_item(&cur, &mut items)?;
    Ok(items)
}

fn unknown_field_msg(field: &str) -> String {
    let expected: Vec<String> = MANIFEST_FIELDS.iter().map(|f| format!("`{}`", f)).collect();
    format!(
        "unknown field `{}`, expected one of {}",
        field,
        expected.join(", ")
    )
}

fn validate_map(map: &[(String, String)]) -> Result<(), String> {
    let mut seen: Vec<&str> = Vec::new();
    let mut name_value: Option<String> = None;
    for (key, value) in map {
        if !MANIFEST_FIELDS.contains(&key.as_str()) {
            return Err(unknown_field_msg(key));
        }
        if seen.contains(&key.as_str()) {
            return Err(format!("duplicate entry with key `{}`", key));
        }
        seen.push(key.as_str());
        if key == "name" {
            name_value = Some(value.clone());
        }
    }
    let name_value = match name_value {
        Some(v) => v,
        None => return Err("missing field `name`".to_string()),
    };
    match classify_scalar(&name_value) {
        Scalar::Null => Err("invalid type: unit value, expected a string".to_string()),
        Scalar::Bool => Err(format!(
            "invalid type: boolean `{}`, expected a string",
            name_value.trim()
        )),
        Scalar::Integer(n) => Err(format!("invalid type: integer `{}`, expected a string", n)),
        Scalar::Float(f) => Err(format!(
            "invalid type: floating point `{}`, expected a string",
            f
        )),
        Scalar::Str => Ok(()),
    }
}

fn scalar_document_msg(raw: &str) -> String {
    let t = raw.trim();
    match classify_scalar(t) {
        Scalar::Null => "invalid type: unit value, expected struct Manifest".to_string(),
        Scalar::Bool => format!("invalid type: boolean `{}`, expected struct Manifest", t),
        Scalar::Integer(n) => format!("invalid type: integer `{}`, expected struct Manifest", n),
        Scalar::Float(f) => format!(
            "invalid type: floating point `{}`, expected struct Manifest",
            f
        ),
        Scalar::Str => "invalid type: string, expected struct Manifest".to_string(),
    }
}

fn validate_manifest(content: &str) -> Result<(), String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut idx = 0;
    while idx < lines.len() {
        let t = lines[idx].trim();
        if t.is_empty() || t.starts_with('#') || t == "---" {
            idx += 1;
            continue;
        }
        break;
    }
    if idx >= lines.len() {
        return Err("missing field `name`".to_string());
    }
    let first = lines[idx].trim();
    if first == "..." {
        return Err("missing field `name`".to_string());
    }
    if first.starts_with("- ") || first == "-" || first.starts_with('[') {
        return Err("invalid type: sequence, expected struct Manifest".to_string());
    }
    if first.starts_with('{') {
        let map = parse_flow_map(first)?;
        return validate_map(&map);
    }
    if split_key_value(first).is_some() {
        let mut map: Vec<(String, String)> = Vec::new();
        let mut i = idx;
        while i < lines.len() {
            let raw = lines[i];
            let t = raw.trim();
            if t.is_empty() || t.starts_with('#') {
                i += 1;
                continue;
            }
            if t == "---" || t == "..." {
                break;
            }
            if raw.starts_with(' ') || raw.starts_with('\t') {
                i += 1;
                continue;
            }
            match split_key_value(raw) {
                Some(kv) => map.push(kv),
                None => {
                    return Err("did not find expected ':' while scanning a simple key".to_string())
                }
            }
            i += 1;
        }
        return validate_map(&map);
    }
    Err(scalar_document_msg(first))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_case(args: &[&str], files: &[(&str, &str)]) -> (i32, String, String) {
        let owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let mut out: Vec<u8> = Vec::new();
        let mut err: Vec<u8> = Vec::new();
        let code = run(
            &owned,
            |path| {
                for (p, content) in files {
                    if *p == path {
                        return Ok((*content).to_string());
                    }
                }
                Err(io::Error::from_raw_os_error(2))
            },
            &mut out,
            &mut err,
        );
        (
            code,
            String::from_utf8(out).expect("stdout must be UTF-8"),
            String::from_utf8(err).expect("stderr must be UTF-8"),
        )
    }

    fn not_found_stderr() -> String {
        format!(
            "unable to read manifest: {}\n",
            io::Error::from_raw_os_error(2)
        )
    }

    #[test]
    fn no_args_prints_usage_on_stderr_with_exit_2() {
        let (code, out, err) = run_case(&[], &[]);
        assert_eq!(code, 2);
        assert_eq!(out, "");
        assert_eq!(
            err,
            "Usage: mote [--jsonl] <manifest.yaml> <task>\n       mote --version\n"
        );
    }

    #[test]
    fn help_prints_usage_on_stdout_with_exit_0() {
        let (code, out, err) = run_case(&["--help"], &[]);
        assert_eq!(code, 0);
        assert_eq!(
            out,
            "Usage: mote [--jsonl] <manifest.yaml> <task>\n       mote --version\n"
        );
        assert_eq!(err, "");
    }

    #[test]
    fn short_help_matches_long_help() {
        let a = run_case(&["--help"], &[]);
        let b = run_case(&["-h"], &[]);
        assert_eq!(a, b);
        assert_eq!(b.0, 0);
    }

    #[test]
    fn version_prints_exact_observed_version_line() {
        let (code, out, err) = run_case(&["--version"], &[]);
        assert_eq!(code, 0);
        assert_eq!(out, "mote 0.13.1\n");
        assert_eq!(err, "");
    }

    #[test]
    fn short_v_is_rejected_like_a_usage_error() {
        let (code, out, err) = run_case(&["-V"], &[]);
        assert_eq!(code, 2);
        assert_eq!(out, "");
        assert_eq!(
            err,
            "Usage: mote [--jsonl] <manifest.yaml> <task>\n       mote --version\n"
        );
    }

    #[test]
    fn jsonl_alone_is_a_usage_error() {
        let (code, out, err) = run_case(&["--jsonl"], &[]);
        assert_eq!(code, 2);
        assert_eq!(out, "");
        assert!(err.starts_with("Usage: mote"));
    }

    #[test]
    fn single_positional_is_a_usage_error() {
        let (code, out, err) = run_case(&["no-such-manifest.yaml"], &[]);
        assert_eq!(code, 2);
        assert_eq!(out, "");
        assert!(err.starts_with("Usage: mote"));
    }

    #[test]
    fn unknown_flag_is_a_usage_error() {
        let (code, out, err) = run_case(&["--unexpected"], &[]);
        assert_eq!(code, 2);
        assert_eq!(out, "");
        assert_eq!(
            err,
            "Usage: mote [--jsonl] <manifest.yaml> <task>\n       mote --version\n"
        );
    }

    #[test]
    fn missing_manifest_with_task_reports_read_error() {
        let (code, out, err) = run_case(&["no-such-manifest.yaml", "hello"], &[]);
        assert_eq!(code, 2);
        assert_eq!(out, "");
        assert_eq!(err, not_found_stderr());
    }

    #[test]
    fn jsonl_flag_does_not_change_read_error() {
        let plain = run_case(&["no-such-manifest.yaml", "hello"], &[]);
        let with_flag = run_case(&["--jsonl", "no-such-manifest.yaml", "hello"], &[]);
        assert_eq!(plain, with_flag);
        assert_eq!(with_flag.0, 2);
        assert_eq!(with_flag.2, not_found_stderr());
    }

    #[test]
    fn empty_manifest_is_missing_name_field() {
        let (code, out, err) = run_case(&["empty.yaml", "hello"], &[("empty.yaml", "")]);
        assert_eq!(code, 2);
        assert_eq!(out, "");
        assert_eq!(err, "invalid manifest: missing field `name`\n");
    }

    #[test]
    fn bracket_document_is_rejected_as_sequence() {
        let (code, out, err) = run_case(&["invalid.yaml", "hello"], &[("invalid.yaml", "[")]);
        assert_eq!(code, 2);
        assert_eq!(out, "");
        assert_eq!(
            err,
            "invalid manifest: invalid type: sequence, expected struct Manifest\n"
        );
    }

    #[test]
    fn null_document_is_rejected_as_unit_value() {
        let (code, out, err) = run_case(&["null.yaml", "hello"], &[("null.yaml", "null\n")]);
        assert_eq!(code, 2);
        assert_eq!(out, "");
        assert_eq!(
            err,
            "invalid manifest: invalid type: unit value, expected struct Manifest\n"
        );
    }

    #[test]
    fn block_sequence_is_rejected_as_sequence() {
        let (code, out, err) = run_case(
            &["array.yaml", "hello"],
            &[("array.yaml", "- one\n- two\n")],
        );
        assert_eq!(code, 2);
        assert_eq!(out, "");
        assert_eq!(
            err,
            "invalid manifest: invalid type: sequence, expected struct Manifest\n"
        );
    }

    #[test]
    fn unknown_field_lists_all_expected_fields_in_order() {
        let (code, out, err) = run_case(
            &["other.yaml", "hello"],
            &[("other.yaml", "not_a_field: true\n")],
        );
        assert_eq!(code, 2);
        assert_eq!(out, "");
        assert_eq!(
            err,
            "invalid manifest: unknown field `not_a_field`, expected one of `name`, `workspace`, `capabilities`, `max_iterations`, `max_per_tool`, `max_runtime_seconds`, `command_timeout_seconds`, `max_output_bytes`, `shell`, `output_file`, `models`, `decision`\n"
        );
    }

    #[test]
    fn help_with_extra_arguments_still_prints_help() {
        let (code, out, err) = run_case(&["--help", "extra"], &[]);
        assert_eq!(code, 0);
        assert_eq!(
            out,
            "Usage: mote [--jsonl] <manifest.yaml> <task>\n       mote --version\n"
        );
        assert_eq!(err, "");
    }

    #[test]
    fn version_with_extra_arguments_still_prints_version() {
        let (code, out, err) = run_case(&["--version", "extra"], &[]);
        assert_eq!(code, 0);
        assert_eq!(out, "mote 0.13.1\n");
        assert_eq!(err, "");
    }

    #[test]
    fn valid_manifest_is_admitted_but_runtime_is_not_implemented() {
        let (code, out, err) = run_case(
            &["manifest.yaml", "do something"],
            &[("manifest.yaml", "name: demo\n")],
        );
        assert_eq!(code, 1);
        assert_eq!(out, "");
        assert!(err.contains("not implemented"), "stderr was: {}", err);
        assert!(!err.to_lowercase().contains("success"));
    }

    #[test]
    fn manifest_without_name_is_missing_field() {
        let (code, _, err) = run_case(
            &["manifest.yaml", "task"],
            &[("manifest.yaml", "workspace: /tmp\n")],
        );
        assert_eq!(code, 2);
        assert_eq!(err, "invalid manifest: missing field `name`\n");
    }

    #[test]
    fn name_must_be_a_scalar_string() {
        let (code, _, err) = run_case(
            &["manifest.yaml", "task"],
            &[("manifest.yaml", "name:\n")],
        );
        assert_eq!(code, 2);
        assert_eq!(
            err,
            "invalid manifest: invalid type: unit value, expected a string\n"
        );
    }

    #[test]
    fn unknown_dash_argument_with_task_is_a_manifest_path() {
        let (code, out, err) = run_case(&["--unexpected", "hello"], &[]);
        assert_eq!(code, 2);
        assert_eq!(out, "");
        assert_eq!(err, not_found_stderr());
    }

    #[test]
    fn extra_positionals_are_ignored_after_missing_manifest_read() {
        let (code, out, err) = run_case(&["no-such-manifest.yaml", "hello", "extra"], &[]);
        assert_eq!(code, 2);
        assert_eq!(out, "");
        assert_eq!(err, not_found_stderr());
    }

    #[test]
    fn repeated_jsonl_flag_is_still_a_usage_error() {
        let (code, out, err) = run_case(&["--jsonl", "--jsonl"], &[]);
        assert_eq!(code, 2);
        assert_eq!(out, "");
        assert!(err.starts_with("Usage: mote"));
    }

    #[test]
    fn help_takes_precedence_over_later_flags() {
        let (code, out, err) = run_case(&["--jsonl", "--help", "--version"], &[]);
        assert_eq!(code, 0);
        assert!(out.starts_with("Usage: mote"));
        assert_eq!(err, "");
    }
}
