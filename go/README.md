# mote (offline CLI reconstruction)

A Go reconstruction of the executable-observed offline CLI behavior of
`mote 0.13.1`, built only from the decoded observations in
`evidence/observations-decoded.md` and `evidence/public-observations.json`.

## Scope

This package implements **only** offline argument handling and
malformed/missing-manifest admission:

- `--help` / `-h` prints usage to stdout and exits 0.
- `--version` prints `mote 0.13.1` to stdout and exits 0.
- `--jsonl` is accepted as a flag and does not change admission errors.
- Help/version are honoured in left-to-right argument order (first wins).
- Unknown dash-prefixed tokens are treated as positional arguments, matching
  the observed examples.
- Fewer than two positionals prints the usage to stderr and exits 2.
- A manifest is read from the first positional; a read failure reports the OS
  error and exits 2.
- A minimal YAML admission check reproduces the observed
  `invalid manifest: ...` messages for missing fields, unknown fields and
  wrong top-level types.

## Not implemented

The MOTE **runtime is not implemented**: no model calls, no tool execution and
no agent loop. A manifest that passes admission does not produce a fabricated
agent result; the process fails honestly with a non-zero exit status and an
explicit message on stderr. This reconstruction is offline and uses the Go
standard library only, with no downloads, external processes or runtime
dependencies.

## Layout

- `go.mod` — standard-library-only module.
- `main.go` — CLI entry point and admission logic.
- `main_test.go` — native tests grounded in the decoded examples.
