package main

import (
	"bytes"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"testing"
)

// Decoded constants taken from evidence/observations-decoded.md.
const (
	evUsage   = "Usage: mote [--jsonl] <manifest.yaml> <task>\n       mote --version\n"
	evVersion = "mote 0.13.1\n"

	evMissingFile = "unable to read manifest: The system cannot find the file specified. (os error 2)\n"

	evMissingName = "invalid manifest: missing field `name`\n"
	evSequence    = "invalid manifest: invalid type: sequence, expected struct Manifest\n"
	evUnit        = "invalid manifest: invalid type: unit value, expected struct Manifest\n"
	evUnknown     = "invalid manifest: unknown field `unexpected_field`, expected one of `name`, `workspace`, `capabilities`, `max_iterations`, `max_per_tool`, `max_runtime_seconds`, `command_timeout_seconds`, `max_output_bytes`, `shell`, `output_file`, `models`, `decision`\n"
)

func runCase(t *testing.T, args []string) (int, string, string) {
	t.Helper()
	var stdout, stderr bytes.Buffer
	code := run(args, &stdout, &stderr)
	return code, stdout.String(), stderr.String()
}

func writeFixture(t *testing.T, name, content string) string {
	t.Helper()
	dir := t.TempDir()
	path := filepath.Join(dir, name)
	if err := os.WriteFile(path, []byte(content), 0o644); err != nil {
		t.Fatalf("write fixture %s: %v", name, err)
	}
	return path
}

func TestUsageAndHelpControls(t *testing.T) {
	cases := []struct {
		name string
		args []string
		code int
		out  string
		err  string
	}{
		{"no-args", []string{}, 2, "", evUsage},
		{"help", []string{"--help"}, 0, evUsage, ""},
		{"short-help", []string{"-h"}, 0, evUsage, ""},
		{"help-with-extra", []string{"--help", "extra"}, 0, evUsage, ""},
		{"short-help-with-file", []string{"-h", "empty.yaml"}, 0, evUsage, ""},
		{"jsonl-help", []string{"--jsonl", "--help"}, 0, evUsage, ""},
		{"version", []string{"--version"}, 0, evVersion, ""},
		{"version-with-extra", []string{"--version", "extra"}, 0, evVersion, ""},
		{"version-then-jsonl", []string{"--version", "--jsonl"}, 0, evVersion, ""},
		{"version-before-help", []string{"--version", "--help"}, 0, evVersion, ""},
		{"help-before-version", []string{"--help", "--version"}, 0, evUsage, ""},
		{"dash-capital-v-unknown", []string{"-V"}, 2, "", evUsage},
		{"jsonl-only", []string{"--jsonl"}, 2, "", evUsage},
		{"jsonl-twice", []string{"--jsonl", "--jsonl"}, 2, "", evUsage},
		{"unknown-flag-only", []string{"--unexpected"}, 2, "", evUsage},
		{"manifest-only", []string{"no-such-manifest.yaml"}, 2, "", evUsage},
		{"empty-file-only", []string{"empty.yaml"}, 2, "", evUsage},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			code, out, errOut := runCase(t, tc.args)
			if code != tc.code || out != tc.out || errOut != tc.err {
				t.Fatalf("args=%q\n got code=%d out=%q err=%q\nwant code=%d out=%q err=%q",
					tc.args, code, out, errOut, tc.code, tc.out, tc.err)
			}
		})
	}
}

func TestManifestReadFailure(t *testing.T) {
	if runtime.GOOS != "windows" {
		t.Skipf("decoded OS error string is Windows-specific")
	}
	cases := [][]string{
		{"no-such-manifest.yaml", "hello"},
		{"--jsonl", "no-such-manifest.yaml", "hello"},
		{"no-such-manifest.yaml", "hello", "extra"},
		{"--unexpected", "hello"},
	}
	for _, args := range cases {
		t.Run(strings.Join(args, "_"), func(t *testing.T) {
			code, out, errOut := runCase(t, args)
			if code != 2 || out != "" || errOut != evMissingFile {
				t.Fatalf("args=%q got code=%d out=%q err=%q", args, code, out, errOut)
			}
		})
	}
}

func TestManifestAdmissionFixtures(t *testing.T) {
	cases := []struct {
		name    string
		file    string
		content string
		want    string
	}{
		{"empty", "empty.yaml", "", evMissingName},
		{"invalid-flow-seq", "invalid.yaml", "[", evSequence},
		{"null-unit", "null.yaml", "null\n", evUnit},
		{"block-seq", "array.yaml", "- one\n- two\n", evSequence},
		{"unknown-field", "unknown.yaml", "unexpected_field: true\n", evUnknown},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			path := writeFixture(t, tc.file, tc.content)
			code, out, errOut := runCase(t, []string{path, "hello"})
			if code != 2 || out != "" || errOut != tc.want {
				t.Fatalf("fixture %s got code=%d out=%q err=%q want err=%q",
					tc.file, code, out, errOut, tc.want)
			}
		})
	}
}

func TestOtherUnknownFieldName(t *testing.T) {
	path := writeFixture(t, "other.yaml", "not_a_field: true\n")
	code, out, errOut := runCase(t, []string{path, "hello"})
	want := "invalid manifest: unknown field `not_a_field`, expected one of `name`, `workspace`, `capabilities`, `max_iterations`, `max_per_tool`, `max_runtime_seconds`, `command_timeout_seconds`, `max_output_bytes`, `shell`, `output_file`, `models`, `decision`\n"
	if code != 2 || out != "" || errOut != want {
		t.Fatalf("got code=%d out=%q err=%q want err=%q", code, out, errOut, want)
	}
}

func TestJsonlDoesNotChangeAdmissionErrors(t *testing.T) {
	path := writeFixture(t, "invalid.yaml", "[")
	code, out, errOut := runCase(t, []string{"--jsonl", path, "hello"})
	if code != 2 || out != "" || errOut != evSequence {
		t.Fatalf("got code=%d out=%q err=%q", code, out, errOut)
	}
}
