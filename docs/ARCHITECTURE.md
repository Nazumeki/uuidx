# Architecture

`uuidx` is a two-crate workspace with a one-way dependency:

```text
uuidx-cli  ->  uuidx-core  ->  uuid crate
```

The core crate does not know whether a caller is a terminal, a shell pipeline,
or another Rust program. The CLI crate does not implement UUID bit rules.

## Core crate

`crates/uuidx-core/src/` is organized by domain responsibility:

| Module | Responsibility |
| --- | --- |
| `types.rs` | Public generation/inspection enums, options, variants, and UUID type exports |
| `error.rs` | Domain errors for parsing, hex payloads, and generation options |
| `parse.rs` | UUID text parsing through the `uuid` crate |
| `format.rs` | Canonical, simple, URN, and braced UUID formats plus fixed-size hex payloads |
| `generate/` | Version-specific v4-v8 generation modules |
| `inspect/` | Version classification, RFC variant detection, metadata, and bit layouts |
| `inspect/ulid.rs` | Optional ULID parsing for read-only inspection |

The public generation enum intentionally contains only v4-v8. That makes
unsupported generation a type-level property rather than a runtime fallback.
Inspection uses a separate enum because legacy v1-v3 and reserved Nil/Max data
must still be understood.

The `uuid` crate remains responsible for UUID construction and version-specific
encoding inside the core crate. The public `uuidx_core::Uuid` is a small
read/format wrapper rather than a re-export, so legacy constructors cannot be
reached through the library API. `uuidx-core` adds the product policy around
which constructors are used, how options are validated, and how metadata is
presented.

## CLI crate

`crates/uuidx-cli/src/` is split along the command and presentation boundaries:

| Module | Responsibility |
| --- | --- |
| `cli/` | Clap argument and subcommand definitions |
| `commands/` | Generate, inspect, validate, and convert workflows |
| `input.rs` | Positional, file, piped stdin, blank-line, and fail-fast handling |
| `output/` | Output mode selection and stream-safe rendering |
| `output/pretty/` | Compact colored sections, semantic themes, and human-readable formatting |
| `output/json.rs` | Stable newline-delimited JSON records |
| `errors.rs` | Process-level error and exit-status mapping |
| `app.rs` | Thin command dispatch and flush boundary |

Commands produce domain values or domain errors and pass them to `Output`. The
renderer decides whether the result is plain, pretty, or JSON. This keeps
terminal styling out of the UUID implementation and prevents ANSI formatting
from leaking into pipeline or JSON output.

## Data flow

```text
arguments / piped stdin / file
          |
          v
      input.rs  ---- invalid record ----> output error record + status 1
          |
          v
      commands/
          |
          v
      uuidx-core
          |
          v
      output mode: plain | pretty | JSONL
```

Top-level usage and I/O failures are kept separate from invalid data records.
That allows a batch command to report every bad record while still returning a
machine-checkable nonzero data status.

## Extension rules

When adding a UUID feature:

1. Put UUID construction or inspection rules in the smallest relevant core
   module.
2. Add a public type or error only when it is part of the caller-facing
   contract.
3. Add a focused core integration test before wiring a CLI option.
4. Keep CLI parsing, input, rendering, and serialization in their existing
   boundaries.
5. Update the JSON contract and documentation when output fields change.

Avoid central registries, compatibility adapters, and speculative plugin
layers. The current dependency direction is enough to support a library caller,
the CLI, and future output consumers without coupling those concerns.

## Development interface

The root `justfile` is the canonical local entry point for formatting, both
feature matrices, linting, release builds, smoke checks, and optional coverage
reports. GitHub Actions runs the equivalent Cargo checks directly, adds the
minimum Rust 1.85 verification and an operating-system feature matrix, and
then performs the release build and smoke checks.
