# MOTE CLI Factory XP

**XP = experiment. This is a limited offline CLI reconstruction, not a full MOTE replacement.**

Ghost Factory dispatched OpenCode workers to generate separate Rust and Go implementations from observations of a pinned MOTE Windows executable. Original MOTE source was not supplied to the workers. The language workers ran in parallel; final integration was serial. A later explicitly authorized Factory publication workflow exported these accepted files without changing their implementation and pushed this repository through a trusted deterministic publication worker.

## What is implemented

- Observed help/version behavior and argument precedence.
- The admitted offline missing/malformed-manifest diagnostics.
- A limited YAML-shape admission check, not a complete YAML parser or MOTE manifest implementation.

**Not implemented:** the agent loop, model/provider calls, tool or shell execution, complete manifest semantics, JSONL runtime event streaming, or general MOTE compatibility. An admitted task fails honestly instead of fabricating an agent result. The output text `mote 0.13.1` is preserved for compatibility with the reference; these are not upstream MOTE releases.

## Source layout

- `rust/` — generated standard-library-only Rust source, native tests and language README.
- `go/` — generated standard-library-only Go source, native tests and language README.
- `evidence/public-observations.json` — public fixtures and exact stdout/stderr/exit vectors.
- `evidence/observations-decoded.md` — readable public observations.
- `scripts/check_public.py` — independent exact-byte public comparator.
- `metadata/ACCEPTANCE.json` — sanitized recorded acceptance/provenance, without private paths or protected test inputs.

## Build and test (Windows)

Clone outside any other Cargo workspace. The qualified host used the stable Windows GNU Rust toolchain and Go 1.25.5. A suitable Rust linker/toolchain and Go are required to build; Python 3 is only needed for the optional public comparator.

```text
cd rust
cargo build --release --offline
cargo test --offline
cd ../go
go build -o mote.exe .
go test ./...
cd ..
python scripts/check_public.py --rust rust/target/release/mote.exe --go go/mote.exe
```

The public vectors are specific to the pinned **Windows** executable and its OS diagnostic strings; other operating systems and broader compatibility are not qualified. Unit passes alone are not black-box compatibility.

## Recorded bounded QA

- Each implementation matched 29/29 public cases and 16/16 operator-held protected cases by exact stdout bytes, stderr bytes and process exit status.
- Rust: 24 native unit tests passed. Go: five top-level tests plus 26 subtests passed, with no failures/skips.
- A wrong-usage mutation in disposable copies was rejected by both native test suites.
- Extracted Windows binaries passed help/version smoke checks without SDK directories on PATH.
- The packaged Rust source also passed offline Cargo release build and tests; those overlap the same native tests and are not counted twice.

The 16 protected cases and raw private run logs are intentionally not published. Their results are recorded operator evidence, not a claim that the public checkout alone reproduces the protected evaluation. Finite matching cases do not establish complete semantic equivalence.

## Provenance and publication boundaries

Accepted local source checkpoint: `626351e61dc80b49d3dc531b485f71ef70509213`.

Reference executable SHA-256: `fcbaf70916a2ab065366b14cd6b6b7d39afc50a59626a41adc8b157d5a3bc92f`.

Generation model: `opencode/deepseek-v4.1-flash-free`. Jack supplied operator inputs and independent QA, not replica implementation fixes. REE supplied a static artifact inventory only; Ghost Research was not used. Observed tool restrictions are not OS-enforced clean-room isolation or hostile-code sandboxing.

Only the accepted generated code, public observations and scoped documentation are published here. No original MOTE executable/source, Factory source/history, protected cases, credentials, runtime homes or raw private logs are included. No hosted CI run, deployment, performance benchmark, full Factory release or automatic general-purpose publishing system is claimed. No license has been added by this publication.
