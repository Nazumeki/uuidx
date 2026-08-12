# Architecture

This is the developer reference for crate ownership, dependency direction,
data flow, and extension boundaries. User installation and source-build
instructions are in [`INSTALLATION.md`](INSTALLATION.md) and
[`SOURCEBUILD.md`](SOURCEBUILD.md); contributor workflow is in
[`CONTRIBUTION.md`](../CONTRIBUTION.md).

`uuidx` is a four-crate workspace with three adapters over one domain crate:

```text
uuidx-cli  ---+
uuidx-wasm ---+--> uuidx-core --> uuid crate
uuidx-ffi  ---+
```

The adapters do not depend on each other. The core crate does not know whether
a caller is a terminal, JavaScript, C, or another Rust program, and no adapter
implements UUID bit rules.

## Core crate

`crates/uuidx-core/src/` is organized by domain responsibility:

| Module                 | Responsibility                                                               |
| ---------------------- | ---------------------------------------------------------------------------- |
| `types.rs`             | Public generation/inspection enums, options, variants, and UUID type exports |
| `error.rs`             | Domain errors for parsing, hex payloads, and generation options              |
| `parse.rs`             | UUID text parsing through the `uuid` crate                                   |
| `format.rs`            | Canonical, simple, URN, and braced UUID formats plus fixed-size hex payloads |
| `generate/`            | Version-specific v3-v8 generation modules                                    |
| `inspect/`             | UUID metadata plus automatic identifier-family recognition                   |
| `inspect/nanoid.rs`    | Standard 21-character NanoID inspection                                      |
| `inspect/snowflake.rs` | Original Twitter Snowflake decoding                                          |
| `inspect/ulid.rs`      | Optional ULID parsing for read-only inspection                               |

The public generation enum intentionally contains only v3-v8. That makes
unsupported generation a type-level property rather than a runtime fallback.
Inspection uses a separate enum because legacy v1-v2 and reserved Nil/Max data
must still be understood.

The `uuid` crate remains responsible for UUID construction and version-specific
encoding inside the core crate. The public `uuidx_core::Uuid` is a small
read/format wrapper rather than a re-export, so legacy constructors cannot be
reached through the library API. `uuidx-core` adds the product policy around
which constructors are used, how options are validated, and how metadata is
presented.

## CLI crate

`crates/uuidx-cli/src/` is split along the command and presentation boundaries:

| Module           | Responsibility                                                           |
| ---------------- | ------------------------------------------------------------------------ |
| `cli/`           | Clap argument and subcommand definitions                                 |
| `commands/`      | Generate, inspect, validate, and convert workflows                       |
| `input.rs`       | Positional, file, piped stdin, blank-line, and fail-fast handling        |
| `output/`        | Output mode selection and stream-safe rendering                          |
| `output/pretty/` | Compact colored sections, semantic themes, and human-readable formatting |
| `output/json.rs` | Stable newline-delimited JSON records                                    |
| `errors.rs`      | Process-level error and exit-status mapping                              |
| `app.rs`         | Thin command dispatch and flush boundary                                 |

Commands produce domain values or domain errors and pass them to `Output`. The
renderer decides whether the result is plain, pretty, or JSON. This keeps
terminal styling out of the UUID implementation and prevents ANSI formatting
from leaking into pipeline or JSON output.

## WebAssembly crate

`uuidx-wasm` exports the core generation, validation, formatting, and
inspection operations through `wasm-bindgen`. It converts JavaScript option
objects and result objects at the boundary, represents wide numeric inspection
fields without losing precision, and maps domain failures to JavaScript errors
with stable codes. Automatic identifier detection is owned by the core so the
CLI and WebAssembly adapters share one recognition order. Optional ULID
inspection follows the core feature gate.

## FFI crate

`uuidx-ffi` exposes a C ABI for the same core operations. It enforces its
null-pointer, length, UTF-8, and value contracts, converts domain results into
ABI-safe values, and owns the error allocation/freeing contract. Its public
header is part of the exported interface and must stay synchronized with the
Rust symbols.

## CLI data flow

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
6. Keep JavaScript conversion and error mapping in `uuidx-wasm`.
7. Keep C layout, ownership, and error handling in `uuidx-ffi`, and update its
   header with every exported ABI change.

Avoid central registries, compatibility adapters, and speculative plugin
layers. The current dependency direction supports Rust, CLI, JavaScript, and C
callers without coupling their presentation and ownership concerns.

## Development interface

The root `justfile` is the canonical local entry point for formatting, both
feature matrices, linting, release builds, smoke checks, and optional coverage
reports. GitHub Actions runs the equivalent Cargo checks directly, adds the
minimum Rust 1.88 verification and an operating-system feature matrix, and
then performs the release build and smoke checks. A dedicated job also compiles
`uuidx-wasm` for `wasm32-unknown-unknown`.
