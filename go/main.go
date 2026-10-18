package main

import (
	"errors"
	"fmt"
	"io"
	"os"
	"strconv"
	"strings"
	"syscall"
)

const usage = "Usage: mote [--jsonl] <manifest.yaml> <task>\n       mote --version\n"

const version = "mote 0.13.1\n"

var manifestFields = []string{
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
}

func main() {
	os.Exit(run(os.Args[1:], os.Stdout, os.Stderr))
}

// run reconstructs the executable-observed offline CLI behavior. It returns
// the process exit status and writes exactly the observed bytes to stdout and
// stderr. It never executes a reference binary or delegates output.
func run(args []string, stdout, stderr io.Writer) int {
	var positionals []string
	jsonl := false

	for _, arg := range args {
		switch arg {
		case "--help", "-h":
			io.WriteString(stdout, usage)
			return 0
		case "--version":
			io.WriteString(stdout, version)
			return 0
		case "--jsonl":
			jsonl = true
		default:
			positionals = append(positionals, arg)
		}
	}
	_ = jsonl

	if len(positionals) < 2 {
		io.WriteString(stderr, usage)
		return 2
	}

	manifestPath := positionals[0]

	data, err := os.ReadFile(manifestPath)
	if err != nil {
		fmt.Fprintf(stderr, "unable to read manifest: %s\n", osErrorString(err))
		return 2
	}

	if msg := validateManifest(data); msg != "" {
		fmt.Fprintf(stderr, "invalid manifest: %s\n", msg)
		return 2
	}

	// Manifest admission succeeded. The agent runtime, model calls and tool
	// execution are intentionally not implemented by this offline
	// reconstruction, so fail honestly rather than fabricate an agent result.
	io.WriteString(stderr, "unable to run task: agent runtime is not implemented\n")
	return 2
}

// osErrorString renders an OS error using the same shape observed from the
// reference executable: "<os message> (os error <n>)".
func osErrorString(err error) string {
	var errno syscall.Errno
	if errors.As(err, &errno) {
		return fmt.Sprintf("%s (os error %d)", errno.Error(), int(errno))
	}
	return err.Error()
}

const (
	kindMap = iota
	kindSeq
	kindUnit
	kindScalar
)

// validateManifest performs the minimal manifest admission observed from the
// reference binary. It returns the message that follows "invalid manifest: "
// or an empty string when the document is an admitted mapping with a name.
func validateManifest(data []byte) string {
	kind, keys := scanTopLevel(data)
	switch kind {
	case kindSeq:
		return "invalid type: sequence, expected struct Manifest"
	case kindUnit:
		return "invalid type: unit value, expected struct Manifest"
	case kindScalar:
		return scalarMessage(firstContentLine(data))
	}

	for _, key := range keys {
		if !isKnownField(key) {
			return fmt.Sprintf("unknown field `%s`, expected one of %s", key, expectedFields())
		}
	}
	for _, required := range manifestFields {
		if !containsKey(keys, required) {
			return fmt.Sprintf("missing field `%s`", required)
		}
	}
	return ""
}

func isKnownField(key string) bool {
	for _, f := range manifestFields {
		if f == key {
			return true
		}
	}
	return false
}

func containsKey(keys []string, want string) bool {
	for _, k := range keys {
		if k == want {
			return true
		}
	}
	return false
}

func expectedFields() string {
	parts := make([]string, len(manifestFields))
	for i, f := range manifestFields {
		parts[i] = "`" + f + "`"
	}
	return strings.Join(parts, ", ")
}

// scanTopLevel classifies the document and, for mappings, collects the
// top-level (column-zero) keys in source order.
func scanTopLevel(data []byte) (int, []string) {
	text := strings.TrimPrefix(string(data), "\ufeff")
	lines := strings.Split(text, "\n")
	started := false
	var keys []string

	for _, raw := range lines {
		line := strings.TrimRight(raw, "\r")
		trimmed := strings.TrimSpace(line)
		if trimmed == "" || strings.HasPrefix(trimmed, "#") {
			continue
		}
		indented := len(line) > 0 && (line[0] == ' ' || line[0] == '\t')

		if !started {
			started = true
			switch {
			case isSeqStart(trimmed), strings.HasPrefix(trimmed, "["):
				return kindSeq, nil
			case strings.HasPrefix(trimmed, "{"):
				return kindMap, parseFlowKeys(trimmed)
			case isNullLiteral(trimmed):
				return kindUnit, nil
			}
			if key, ok := mappingKey(trimmed); ok {
				keys = append(keys, key)
				continue
			}
			return kindScalar, nil
		}

		if !indented {
			if key, ok := mappingKey(trimmed); ok {
				keys = append(keys, key)
			}
		}
	}

	if !started {
		return kindMap, nil
	}
	return kindMap, keys
}

func firstContentLine(data []byte) string {
	text := strings.TrimPrefix(string(data), "\ufeff")
	for _, raw := range strings.Split(text, "\n") {
		trimmed := strings.TrimSpace(strings.TrimRight(raw, "\r"))
		if trimmed == "" || strings.HasPrefix(trimmed, "#") {
			continue
		}
		return trimmed
	}
	return ""
}

func isSeqStart(trimmed string) bool {
	if trimmed == "" || trimmed[0] != '-' {
		return false
	}
	if len(trimmed) == 1 {
		return true
	}
	return trimmed[1] == ' ' || trimmed[1] == '\t'
}

func isNullLiteral(trimmed string) bool {
	switch trimmed {
	case "null", "Null", "NULL", "~":
		return true
	}
	return false
}

func mappingKey(line string) (string, bool) {
	idx := strings.IndexByte(line, ':')
	if idx < 0 {
		return "", false
	}
	key := strings.TrimSpace(line[:idx])
	if key == "" {
		return "", false
	}
	if len(key) >= 2 {
		if (key[0] == '"' && key[len(key)-1] == '"') || (key[0] == '\'' && key[len(key)-1] == '\'') {
			key = key[1 : len(key)-1]
		}
	}
	if strings.ContainsAny(key, " \t") {
		return "", false
	}
	return key, true
}

func parseFlowKeys(s string) []string {
	s = strings.TrimSpace(s)
	s = strings.TrimPrefix(s, "{")
	s = strings.TrimSuffix(s, "}")
	var keys []string
	for _, part := range strings.Split(s, ",") {
		part = strings.TrimSpace(part)
		if part == "" {
			continue
		}
		if key, ok := mappingKey(part); ok {
			keys = append(keys, key)
		}
	}
	return keys
}

func scalarMessage(trimmed string) string {
	switch trimmed {
	case "true", "True", "TRUE":
		return "invalid type: boolean `true`, expected struct Manifest"
	case "false", "False", "FALSE":
		return "invalid type: boolean `false`, expected struct Manifest"
	}
	if _, err := strconv.ParseInt(trimmed, 10, 64); err == nil {
		return fmt.Sprintf("invalid type: integer `%s`, expected struct Manifest", trimmed)
	}
	if _, err := strconv.ParseFloat(trimmed, 64); err == nil {
		return fmt.Sprintf("invalid type: floating point `%s`, expected struct Manifest", trimmed)
	}
	return fmt.Sprintf("invalid type: string %q, expected struct Manifest", trimmed)
}
