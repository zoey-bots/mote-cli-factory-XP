# mote — offline CLI admission pilot (Rust)

Standalone reconstruction of the observed **offline CLI admission surface** of the
MOTE Windows executable, written from public black-box observations only. No
original MOTE source was recompiled or inspected, and this crate has **no external
dependencies, build scripts, generators or downloads** — standard library only.

## Build and test

```
cargo build --release
cargo test
```

The binary is `mote` (`mote.exe` on Windows). Output is UTF-8 with LF line
endings; usage/help/version go to stdout for successful requests and errors go to
stderr.

## Commands

```
mote <manifest.yaml> <task>   admit a manifest and (in the original) run a task
mote --jsonl <manifest> <task>  same, requesting JSONL output (flag only here)
mote --help | -h              print usage to stdout, exit 0
mote --version                print "mote 0.13.1" to stdout, exit 0
```

The exact observed version text is preserved verbatim: `mote 0.13.1`.

## Behavior (exit codes)

- `0` — `--help`/`-h` or `--version` was requested (help/version win whenever
  they appear in argv, before any other handling).
- `2` — usage or admission failure:
  - no arguments, fewer than two positionals, or an unknown `-`-prefixed
    argument that ends up alone → usage text on stderr;
  - unreadable manifest → `unable to read manifest: <io error>` on stderr;
  - malformed/unsupported manifest → `invalid manifest: <reason>` on stderr
    (missing `name`, unknown field with the full expected-field list, wrong
    top-level type such as sequence/unit).
- `1` — manifest admitted but the runtime is not available (see below).

Admission logic is general: arguments are parsed by rules, the manifest path is
read from the filesystem, and a limited YAML-shape check (comments, blank lines,
block/flow mappings, sequences, scalars, null) decides validity. There is no
exact-argv lookup table.

Options are recognized only when they are exactly `--jsonl`, `--help`, `-h` or
`--version`. Any other dash-prefixed argument (for example `--unexpected` or
`-V`) is treated as an ordinary positional, matching the observations: it is a
usage error only when it leaves fewer than two positionals. Extra positionals
beyond the manifest and task are ignored. `--help`/`-h`/`--version` win as soon
as they are encountered, scanning left to right.

## Limitations — runtime NOT implemented

**This pilot does NOT implement the agent runtime, model calls, tool execution,
shell execution, iteration loops, output capture, JSONL event streaming, or
network access of any kind.** When a manifest passes admission and a task is
present, the program prints an explicit "not implemented" message to stderr and
exits non-zero (code 1). It never fabricates successful agent results or fake
task output. YAML support is a limited admission check, not full YAML
validation, and unsupported or unknown runtime behavior is refused rather than
simulated.
