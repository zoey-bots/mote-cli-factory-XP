# Decoded public observations

Finite Windows offline CLI/admission reconstruction only; not MOTE model/tool runtime or general semantic equivalence

Reference SHA-256: `fcbaf70916a2ab065366b14cd6b6b7d39afc50a59626a41adc8b157d5a3bc92f`.

## regression-01

Arguments: `[]`. Exit: `2`.

stdout:
```text

```
stderr:
```text
Usage: mote [--jsonl] <manifest.yaml> <task>
       mote --version

```

## regression-02

Arguments: `["--help"]`. Exit: `0`.

stdout:
```text
Usage: mote [--jsonl] <manifest.yaml> <task>
       mote --version

```
stderr:
```text

```

## regression-03

Arguments: `["-h"]`. Exit: `0`.

stdout:
```text
Usage: mote [--jsonl] <manifest.yaml> <task>
       mote --version

```
stderr:
```text

```

## regression-04

Arguments: `["--version"]`. Exit: `0`.

stdout:
```text
mote 0.13.1

```
stderr:
```text

```

## regression-05

Arguments: `["-V"]`. Exit: `2`.

stdout:
```text

```
stderr:
```text
Usage: mote [--jsonl] <manifest.yaml> <task>
       mote --version

```

## regression-06

Arguments: `["--jsonl"]`. Exit: `2`.

stdout:
```text

```
stderr:
```text
Usage: mote [--jsonl] <manifest.yaml> <task>
       mote --version

```

## regression-07

Arguments: `["no-such-manifest.yaml"]`. Exit: `2`.

stdout:
```text

```
stderr:
```text
Usage: mote [--jsonl] <manifest.yaml> <task>
       mote --version

```

## regression-08

Arguments: `["no-such-manifest.yaml", "hello"]`. Exit: `2`.

stdout:
```text

```
stderr:
```text
unable to read manifest: The system cannot find the file specified. (os error 2)

```

## regression-09

Arguments: `["--jsonl", "no-such-manifest.yaml", "hello"]`. Exit: `2`.

stdout:
```text

```
stderr:
```text
unable to read manifest: The system cannot find the file specified. (os error 2)

```

## regression-10

Arguments: `["--unexpected"]`. Exit: `2`.

stdout:
```text

```
stderr:
```text
Usage: mote [--jsonl] <manifest.yaml> <task>
       mote --version

```

## regression-11

Arguments: `["empty.yaml", "hello"]`. Exit: `2`.

stdout:
```text

```
stderr:
```text
invalid manifest: missing field `name`

```

## regression-12

Arguments: `["invalid.yaml", "hello"]`. Exit: `2`.

stdout:
```text

```
stderr:
```text
invalid manifest: invalid type: sequence, expected struct Manifest

```

## regression-13

Arguments: `["null.yaml", "hello"]`. Exit: `2`.

stdout:
```text

```
stderr:
```text
invalid manifest: invalid type: unit value, expected struct Manifest

```

## regression-14

Arguments: `["array.yaml", "hello"]`. Exit: `2`.

stdout:
```text

```
stderr:
```text
invalid manifest: invalid type: sequence, expected struct Manifest

```

## regression-15

Arguments: `["--help", "extra"]`. Exit: `0`.

stdout:
```text
Usage: mote [--jsonl] <manifest.yaml> <task>
       mote --version

```
stderr:
```text

```

## regression-16

Arguments: `["--version", "extra"]`. Exit: `0`.

stdout:
```text
mote 0.13.1

```
stderr:
```text

```

## regression-17

Arguments: `["other.yaml", "hello"]`. Exit: `2`.

stdout:
```text

```
stderr:
```text
invalid manifest: unknown field `not_a_field`, expected one of `name`, `workspace`, `capabilities`, `max_iterations`, `max_per_tool`, `max_runtime_seconds`, `command_timeout_seconds`, `max_output_bytes`, `shell`, `output_file`, `models`, `decision`

```

## regression-18

Arguments: `["--jsonl", "--help"]`. Exit: `0`.

stdout:
```text
Usage: mote [--jsonl] <manifest.yaml> <task>
       mote --version

```
stderr:
```text

```

## regression-19

Arguments: `["--jsonl", "--version"]`. Exit: `0`.

stdout:
```text
mote 0.13.1

```
stderr:
```text

```

## regression-20

Arguments: `["--help", "--version"]`. Exit: `0`.

stdout:
```text
Usage: mote [--jsonl] <manifest.yaml> <task>
       mote --version

```
stderr:
```text

```

## regression-21

Arguments: `["--version", "--help"]`. Exit: `0`.

stdout:
```text
mote 0.13.1

```
stderr:
```text

```

## regression-22

Arguments: `["-h", "empty.yaml"]`. Exit: `0`.

stdout:
```text
Usage: mote [--jsonl] <manifest.yaml> <task>
       mote --version

```
stderr:
```text

```

## regression-23

Arguments: `["no-such-manifest.yaml", "hello", "extra"]`. Exit: `2`.

stdout:
```text

```
stderr:
```text
unable to read manifest: The system cannot find the file specified. (os error 2)

```

## regression-24

Arguments: `["empty.yaml"]`. Exit: `2`.

stdout:
```text

```
stderr:
```text
Usage: mote [--jsonl] <manifest.yaml> <task>
       mote --version

```

## regression-25

Arguments: `["unknown.yaml", "hello"]`. Exit: `2`.

stdout:
```text

```
stderr:
```text
invalid manifest: unknown field `unexpected_field`, expected one of `name`, `workspace`, `capabilities`, `max_iterations`, `max_per_tool`, `max_runtime_seconds`, `command_timeout_seconds`, `max_output_bytes`, `shell`, `output_file`, `models`, `decision`

```

## regression-26

Arguments: `["--jsonl", "invalid.yaml", "hello"]`. Exit: `2`.

stdout:
```text

```
stderr:
```text
invalid manifest: invalid type: sequence, expected struct Manifest

```

## regression-27

Arguments: `["--unexpected", "hello"]`. Exit: `2`.

stdout:
```text

```
stderr:
```text
unable to read manifest: The system cannot find the file specified. (os error 2)

```

## regression-28

Arguments: `["--jsonl", "--jsonl"]`. Exit: `2`.

stdout:
```text

```
stderr:
```text
Usage: mote [--jsonl] <manifest.yaml> <task>
       mote --version

```

## regression-29

Arguments: `["--version", "--jsonl"]`. Exit: `0`.

stdout:
```text
mote 0.13.1

```
stderr:
```text

```
