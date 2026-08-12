# uuidx

`uuidx` is a Rust workspace for generating, inspecting, validating, and
formatting UUIDs. It separates UUID domain behavior from delivery concerns so
the same policy can be used by the `uuidx` command, Rust applications through
`uuidx-core`, JavaScript through `uuidx-wasm`, and native applications through
the `uuidx-ffi` C ABI.

The project is intentionally opinionated about generation. It generates UUID
versions that are useful for current applications, while keeping legacy and
reserved values available for inspection and migration work. Unsupported
generation paths are rejected by the core API and by the CLI.

## Capabilities

- Generate UUID v3, v4, v5, v6, v7, and v8 values with version-specific options.
- Inspect UUID versions, variants, bytes, timestamps, node metadata, name hash
  algorithms, custom payloads, and bit layouts.
- Validate UUID text in batches without changing the generation policy.
- Convert UUIDs between canonical, simple, URN, and braced formats.
- Read values from positional arguments, files, or newline-delimited stdin.
- Produce terminal-oriented output, pipeline-safe plain output, or
  newline-delimited JSON for automation.
- Redact time-based node identifiers while retaining their classification.
- Inspect ULIDs when the optional `ulid-inspect` feature is enabled. ULIDs are
  never generated, validated as UUIDs, or converted by `uuidx`.
- Use the same generation, validation, formatting, and inspection policy from
  JavaScript or native code through focused WebAssembly and C ABI crates.

## UUID support policy

Inspection, validation, and conversion accept syntactically valid UUIDs even
when a version is not a generation target. Inspection reports the UUID variant
as NCS, RFC 9562, Microsoft, or future; validation does not reject a UUID only
because its variant is not RFC 9562.

| UUID family            | Inspect | Validate | Convert | Generate | Behavior                                                                         |
| ---------------------- | ------- | -------- | ------- | -------- | -------------------------------------------------------------------------------- |
| v1                     | Yes     | Yes      | Yes     | No       | Legacy time-based UUID; timestamp, clock sequence, and node data may be exposed. |
| v2                     | Yes     | Yes      | Yes     | No       | DCE Security semantics outside RFC 9562.                                         |
| v3                     | Yes     | Yes      | Yes     | Yes      | Legacy MD5 name hashing; a warning is emitted.                                   |
| v4                     | Yes     | Yes      | Yes     | Yes      | Random UUID.                                                                     |
| v5                     | Yes     | Yes      | Yes     | Yes      | Deterministic namespace/name UUID using SHA-1; a warning is emitted.             |
| v6                     | Yes     | Yes      | Yes     | Yes      | Time-ordered UUID with an optional timestamp and node.                           |
| v7                     | Yes     | Yes      | Yes     | Yes      | Unix-time-ordered UUID with random payload bits.                                 |
| v8                     | Yes     | Yes      | Yes     | Yes      | Application-defined 16-byte payload with UUID version and variant bits applied.  |
| Nil and Max            | Yes     | Yes      | Yes     | No       | Reserved all-zero and all-one values.                                            |
| Unknown version nibble | Yes     | Yes      | Yes     | No       | Parsed and reported as `unknown(n)` for analysis.                                |

Generation details:

- `v4` uses random bytes and accepts no version-specific input.
- `v3` requires both `--namespace` and `--name`. The namespace may be `dns`,
  `url`, `oid`, `x500`, or an explicit UUID. The name is passed as UTF-8
  bytes. MD5 is retained for standards compatibility and is reported as a
  warning.
- `v5` requires both `--namespace` and `--name`. The namespace may be `dns`,
  `url`, `oid`, `x500`, or an explicit UUID. The name is passed as UTF-8
  bytes. SHA-1 is retained for standards compatibility and is reported as a
  warning.
- `v6` accepts an optional RFC 3339 or Unix-millisecond timestamp and a
  six-byte node ID. Without `--node`, it creates a locally generated multicast
  node ID instead of using a hardware-derived node.
- `v7` accepts an optional RFC 3339 or Unix-millisecond timestamp. Without one,
  the current system time is used.
- `v8` requires exactly 16 bytes of hexadecimal application data. The UUID
  version and variant positions are controlled by the UUID encoding, so the
  final value is not a raw copy of every input bit.
- `v1` and `v2` remain inspect-only. UUID v1 exposes host-related metadata, and
  UUID v2 uses DCE Security semantics outside RFC 9562.

## Installation

### Install the CLI from this checkout

The binary package is `uuidx-cli`, and it installs the `uuidx` executable:

```console
cargo install --path crates/uuidx-cli --locked
uuidx --version
```

### Build an optimized binary

```console
cargo build --release --locked --package uuidx-cli --all-features
```

The resulting executable is `target/release/uuidx` on Unix-like systems and
`target\release\uuidx.exe` on Windows.

### Run from the workspace

```console
cargo run --locked --package uuidx-cli -- generate v7
```

The CLI enables `ulid-inspect` by default. To build a smaller feature-disabled
binary, use `--no-default-features`:

```console
cargo build --locked --package uuidx-cli --no-default-features
```

## Quick start

Generate a UUID v7. `generate` defaults to v7 when no version is supplied:

```console
uuidx generate
uuidx generate v7
```

Generate five UUID v4 values as one plain value per line:

```console
uuidx generate v4 --count 5 --output plain
```

Generate a deterministic UUID v5:

```console
uuidx generate v5 --namespace dns --name example.com --output plain
uuidx generate v3 --namespace dns --name example.org --output plain
```

Inspect timestamp and bit-layout metadata:

```console
uuidx inspect 018f2c0b-6c5b-7d2e-8f4a-123456789abc --output pretty --layout
```

Inspect a value as machine-readable JSON:

```console
uuidx inspect 018f2c0b-6c5b-7d2e-8f4a-123456789abc --output json
```

Validate a stream. Valid records are silent in plain mode; invalid records are
reported on standard error and produce exit status `1`:

```bash
printf '%s\n' \
  018f2c0b-6c5b-7d2e-8f4a-123456789abc \
  not-a-uuid | uuidx validate --output plain
```

Convert a UUID to its 32-character simple representation:

```console
uuidx convert 018f2c0b-6c5b-7d2e-8f4a-123456789abc \
  --to simple --output plain
```

## Command reference

The root command is:

```text
uuidx [OPTIONS] <COMMAND>
```

The executable is shown as `uuidx.exe` in help output on Windows. The commands
and their aliases are:

| Command    | Alias | Purpose                                        |
| ---------- | ----- | ---------------------------------------------- |
| `generate` | `g`   | Generate UUID v3-v8 values.                    |
| `inspect`  | `i`   | Inspect UUIDs and optionally recognize ULIDs.  |
| `validate` | `v`   | Validate UUID syntax and report record errors. |
| `convert`  | `c`   | Convert UUID text between standard formats.    |

Global options:

| Option            | Values                            | Description                                                 |
| ----------------- | --------------------------------- | ----------------------------------------------------------- |
| `-v`, `--version` | -                                 | Print the application version. This is a root-level option. |
| `-o`, `--output`  | `auto`, `pretty`, `plain`, `json` | Select the renderer. Default: `auto`.                       |

Use `uuidx <command> --help` for the complete parser-generated help for a
command and its options.

### `generate`

```text
uuidx generate [VERSION] [OPTIONS]
```

`VERSION` accepts `v3`, `v4`, `v5`, `v6`, `v7`, or `v8`, with or without the `v`
prefix. It defaults to `v7`. `--count` must be a positive integer.

| Option                          | Applies to | Description                                                        |
| ------------------------------- | ---------- | ------------------------------------------------------------------ |
| `-n`, `--count <COUNT>`         | All        | Number of values to generate. Default: `1`.                        |
| `-s`, `--namespace <NAMESPACE>` | v3, v5     | Named namespace (`dns`, `url`, `oid`, `x500`) or UUID text.        |
| `-N`, `--name <NAME>`           | v3, v5     | Name text, encoded as UTF-8 bytes.                                 |
| `-d`, `--node <NODE>`           | v6         | Twelve hexadecimal characters representing six bytes, or `random`. |
| `-t`, `--timestamp <TIMESTAMP>` | v6, v7     | RFC 3339 timestamp or non-negative Unix milliseconds.              |
| `-C`, `--custom <HEX>`          | v8         | Exactly 32 hexadecimal characters representing 16 bytes.           |
| `-F`, `--format <FORMAT>`       | All        | `canonical`, `simple`, `urn`, or `braced`. Default: `canonical`.   |

Examples:

```console
uuidx generate v4 --count 3 --output plain
uuidx generate v5 --namespace url --name https://example.com --output plain
uuidx generate v3 --namespace dns --name example.org --output plain
uuidx generate v6 --timestamp 1700000000123 --node 020000000001 --output plain
uuidx generate v7 --timestamp 2024-04-29T22:48:17.243Z --output plain
uuidx generate v8 --custom 00112233445566778899aabbccddeeff --output plain
uuidx generate v4 --format urn --output plain
```

Options are version-scoped. For example, passing `--custom` to v4 or omitting
the required v3/v5 namespace/name pair is a usage error with exit status `2`.

### `inspect`

```text
uuidx inspect [OPTIONS] [VALUE]...
```

Inspection reports the normalized UUID, input format, raw bytes, version,
variant, nil/max flags, and version-specific metadata. With `--layout`, pretty
output also reports field offsets, widths, and hexadecimal values. JSON output
always contains its structured `fields` array; `--layout` only changes the
pretty renderer.

| Option                     | Description                                                          |
| -------------------------- | -------------------------------------------------------------------- |
| `-L`, `--layout`           | Add bit offsets and values to pretty output.                         |
| `-r`, `--redact-sensitive` | Omit time-based node ID bytes while keeping the node classification. |
| `-f`, `--fail-fast`        | Stop after the first invalid input record.                           |
| `-i`, `--input <FILE>`     | Read one value per line from a file instead of positional input.     |

Examples:

```console
uuidx inspect 018f2c0b-6c5b-7d2e-8f4a-123456789abc
uuidx inspect 11111111-1111-6111-9111-111111111111 \
  --redact-sensitive --output json
uuidx inspect 01ARZ3NDEKTSV4RRFFQ69G5FAV --output json
```

ULID inspection is read-only compatibility support. ULID records have
`"kind":"ulid"`, timestamp and random-payload metadata, and an inspection-only
warning. They are not treated as UUIDs by `validate` or `convert`.

### `validate`

```text
uuidx validate [OPTIONS] [VALUE]...
```

`validate` parses UUID text and accepts all syntactically valid UUID versions,
including legacy, nil, max, and unknown-version values. It does not generate
values or enforce the generation allow-list. In plain mode, successful records
write nothing to standard output; JSON mode emits one success record per valid
input. Invalid records are reported with exit status `1`.

```console
uuidx validate 018f2c0b-6c5b-7d2e-8f4a-123456789abc --output json
uuidx validate --input ids.txt --fail-fast --output json
```

### `convert`

```text
uuidx convert [OPTIONS] [VALUE]...
```

`convert` parses UUID text and emits the requested representation. The input
may be canonical, simple, URN, or braced text.

| Option                 | Values                                 | Description                                |
| ---------------------- | -------------------------------------- | ------------------------------------------ |
| `-t`, `--to <FORMAT>`  | `canonical`, `simple`, `urn`, `braced` | Target format. Default: `canonical`.       |
| `-f`, `--fail-fast`    | -                                      | Stop after the first invalid input record. |
| `-i`, `--input <FILE>` | Path                                   | Read one UUID per line from a file.        |

The format aliases `hyphenated`, `hex`, and `brace` are also accepted by the
parser.

```console
uuidx convert 018f2c0b6c5b7d2e8f4a123456789abc \
  --to canonical --output plain
uuidx convert --input ids.txt --to urn --output plain
```

## Input and batch processing

`inspect`, `validate`, and `convert` use the same input contract:

1. Positional values are processed in order.
2. If no positional values are supplied, `--input FILE` reads one value per
   line from the named file.
3. If neither positional values nor `--input` are supplied, the command reads
   newline-delimited values from piped standard input.
4. Leading and trailing whitespace is removed from each record.
5. Blank records are ignored and do not consume a record index.
6. `--fail-fast` stops after the first invalid record. Without it, processing
   continues and the command returns status `1` if any record failed.

`--input` accepts a file path only. `--input -` is rejected; omit `--input` to
read from stdin. When stdin is interactive and no values were supplied, the
command reports a missing-input usage error instead of waiting for input.

JSON record indexes are zero-based and count only non-blank records. A failed
record does not prevent later records from being emitted unless `--fail-fast`
was selected.

## Output modes and JSONL

`--output auto` selects pretty output when standard output is a terminal and
plain output when it is redirected or piped. Choose a mode explicitly when a
script must not depend on terminal detection.

| Mode     | Intended use                         | Behavior                                                                                                                       |
| -------- | ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------ |
| `auto`   | Interactive use and simple pipelines | Pretty on a terminal, plain otherwise.                                                                                         |
| `pretty` | Human diagnosis                      | Labeled summaries for inspection and contextual command output. Colors are enabled only when supported by the output stream.   |
| `plain`  | Shell pipelines                      | One normalized/generated/converted value per successful generate, inspect, or convert record. Successful validation is silent. |
| `json`   | Programs and batch processing        | One newline-delimited JSON object per generated or processed record.                                                           |

Plain and JSON output never include ANSI escape sequences. Record-level data
errors go to standard error in pretty and plain modes. In JSON mode, invalid
records are JSON objects on standard output so a consumer can process the
whole stream; the process still exits with status `1`.

The JSON contract is versioned independently of the application version. Each
record includes `schema_version`, currently `1`, and an `operation` of
`generate`, `inspect`, `validate`, or `convert`. Successful records may include
`value`, `bytes`, `kind`, `version`, `format`, `metadata`, and `fields`. Failed
records include `input` and a structured `error` with `code` and `message`.
Non-fatal policy notices are emitted in `warnings`.

Example inspection record:

```json
{
  "schema_version": 1,
  "operation": "inspect",
  "index": 0,
  "ok": true,
  "input": "018f2c0b-6c5b-7d2e-8f4a-123456789abc",
  "value": "018f2c0b-6c5b-7d2e-8f4a-123456789abc",
  "bytes": "018f2c0b6c5b7d2e8f4a123456789abc",
  "kind": "uuid",
  "version": "v7",
  "metadata": {
    "type": "time",
    "unix_seconds": 1714430897,
    "unix_millis": 1714430897243,
    "subsec_nanos": 243000000,
    "clock_sequence": null,
    "node_id": null,
    "node_kind": null
  },
  "fields": [
    {
      "name": "unix_timestamp_ms",
      "offset": 0,
      "width": 48,
      "value": "0x18f2c0b6c5b"
    }
  ]
}
```

Consumers should branch on `schema_version`, `operation`, and `ok`, and ignore
unknown fields so additive fields can be introduced in a later schema version.
See [`docs/JSON.md`](docs/JSON.md) for the complete field and metadata
reference.

## Exit statuses

| Status | Meaning                                                                                     |
| ------ | ------------------------------------------------------------------------------------------- |
| `0`    | All requested work completed successfully.                                                  |
| `1`    | At least one input record was invalid. The remaining records may still have been processed. |
| `2`    | Usage, argument, missing-input, or generation-option error.                                 |
| `3`    | File, process I/O, output, or JSON serialization failure.                                   |

## Rust library

`uuidx-core` is a separately usable Rust crate in this workspace. It owns the
domain implementation without CLI, terminal, JavaScript, or C ABI dependencies.

The generation enum intentionally contains only v3-v8; inspection uses a
separate enum so legacy and reserved values can still be understood.

The core API includes:

- `parse_uuid` for UUID text parsing.
- `format_uuid` and `UuidOutputFormat` for canonical, simple, URN, and braced
  output.
- `generate_uuid` and `GenerationOptions` for version-specific generation.
- `inspect_uuid` and `UuidInspection` for version, variant, metadata, and bit
  layout information.
- `parse_hex_array` for fixed-size hexadecimal payloads.
- `inspect_ulid` and `UlidInspection` behind the optional `ulid-inspect`
  feature.

Example:

```rust
use uuidx_core::{
    GeneratableUuidVersion, GenerationOptions, generate_uuid, inspect_uuid, parse_uuid,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let uuid = generate_uuid(&GenerationOptions::new(GeneratableUuidVersion::V7))?;
    let inspection = inspect_uuid(&uuid);

    println!("{}", inspection.normalized);
    println!("version: {}", inspection.version);

    let parsed = parse_uuid(&uuid.to_string())?;
    assert_eq!(parsed, uuid);
    Ok(())
}
```

The core crate has no default features. Enable `ulid-inspect` when a Rust
application needs read-only ULID inspection. The dependency source can be a
path, workspace, or registry selected by the deployment:

```toml
[dependencies]
uuidx-core = { version = "0.1", features = ["ulid-inspect"] }
```

## WebAssembly bindings

`uuidx-wasm` exposes JavaScript functions through `wasm-bindgen`:

- `generateUuid(version, options?)` generates and formats v3-v8 UUIDs.
- `validateUuid(input)` reports whether UUID text parses.
- `formatUuid(input, format)` converts UUID text to a supported format.
- `inspectUuid(input)` returns structured UUID metadata and bit fields.
- `inspectUlid(input)` is available with the default `ulid-inspect` feature.

The generated TypeScript declarations define the option and result shapes.
Fallible functions throw JavaScript `Error` values with a machine-readable
`code` property. Inspection uses hexadecimal strings for bit-field values that
may exceed JavaScript's safe integer range.

```javascript
import init, {
  generateUuid,
  inspectUuid,
  validateUuid,
} from "./pkg/uuidx_wasm.js";

await init();

const value = generateUuid("v7", {
  timestampMs: 1714430897243,
  format: "canonical",
});

if (validateUuid(value)) {
  const inspection = inspectUuid(value);
  console.log(inspection.version, inspection.metadata);
}
```

`just build-wasm` verifies the Rust artifact for its supported target. Run
`wasm-bindgen` or a bundler that integrates it to produce the JavaScript module
and declarations consumed above.

## C API

`uuidx-ffi` builds static and dynamic libraries and publishes its C interface
in [`crates/uuidx-ffi/include/uuidx.h`](crates/uuidx-ffi/include/uuidx.h). The
ABI exposes parsing, formatting, named namespaces, v3-v8 generation, and
structured inspection.

```c
#include <stdio.h>
#include <string.h>
#include "uuidx.h"

int main(void) {
    uuidx_uuid_t uuid;
    const char *input = "018f2c0b-6c5b-7d2e-8f4a-123456789abc";
    uuidx_error_t *error = uuidx_uuid_parse(
        (const uint8_t *)input, strlen(input), &uuid
    );

    if (error != NULL) {
        fprintf(stderr, "%s\n", uuidx_error_message(error));
        uuidx_error_free(error);
        return 1;
    }

    return 0;
}
```

Fallible functions return `NULL` on success or one owned `uuidx_error_t` on
failure. Error messages are borrowed until `uuidx_error_free` releases the
error. All UUID, inspection, and formatting buffers are caller-owned; no other
allocations cross the ABI. `uuidx_uuid_format` supports a `NULL`/zero-capacity
size query and reports the required size including the terminating NUL.

## Architecture

The workspace has three adapters over one domain crate:

```text
uuidx-cli  ---+
uuidx-wasm ---+--> uuidx-core --> uuid
uuidx-ffi  ---+
```

`uuidx-core` owns parsing, formatting, generation, inspection, domain errors,
and public policy. `uuidx-cli` owns command-line input and presentation,
`uuidx-wasm` translates the domain API into JavaScript values and errors, and
`uuidx-ffi` owns the C ABI and cross-language memory boundary. The adapters do
not depend on each other.

Repository layout:

```text
crates/
  uuidx-core/
    src/              UUID domain API and version-specific behavior
    tests/            Core integration tests
  uuidx-cli/
    src/              CLI parsing, commands, input, output, and errors
    tests/            End-to-end command tests
  uuidx-wasm/
    src/              WebAssembly exports and JavaScript value conversion
  uuidx-ffi/
    src/              C ABI exports, fixed values, and error ownership
    include/          Public C header
docs/
  ARCHITECTURE.md     Dependency direction and extension rules
  JSON.md             JSONL schema and metadata contract
justfile              Canonical development and CI recipes
```

For the module-level extension rules, see
[`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

## Feature flags

| Package      | Feature        | Default | Effect                                                                    |
| ------------ | -------------- | ------- | ------------------------------------------------------------------------- |
| `uuidx-core` | `ulid-inspect` | No      | Adds read-only ULID parsing and inspection.                               |
| `uuidx-cli`  | `ulid-inspect` | Yes     | Enables the matching core feature and ULID inspection in `uuidx inspect`. |
| `uuidx-wasm` | `ulid-inspect` | Yes     | Exposes read-only ULID inspection to JavaScript.                           |

The feature-disabled build is part of the normal verification matrix. A build
without default features must not silently claim ULID support.

## Development

### Requirements

- Rust `1.88` or newer with Cargo.
- [Just](https://github.com/casey/just) for the repository's standard command
  interface.
- The `wasm32-unknown-unknown` Rust target for `just build-wasm`.
- `wasm-bindgen-test-runner` from `wasm-bindgen-cli` `0.2.127` for
  `just test-wasm-target`; its schema must match the version in `Cargo.lock`.
- `cargo-llvm-cov` only when generating coverage reports.

Install the project, run the full local check, and exercise the CLI with:

```console
just check
just smoke
```

The default `just` recipe is `just check`.

### Common recipes

| Recipe            | Purpose                                                            |
| ----------------- | ------------------------------------------------------------------ |
| `just fmt`        | Format all Rust code.                                              |
| `just fmt-check`  | Verify formatting without modifying files.                         |
| `just test`       | Run workspace tests with all features.                             |
| `just test-min`   | Run workspace tests with default features disabled.                |
| `just test-core`  | Run `uuidx-core` tests with all features.                          |
| `just test-cli`   | Run `uuidx-cli` tests with all features.                           |
| `just test-wasm`  | Run `uuidx-wasm` host tests with all features.                     |
| `just test-wasm-target` | Run WASM exports under Node in both feature modes.          |
| `just test-ffi`   | Run `uuidx-ffi` tests with all features.                           |
| `just test-all`   | Run both feature matrices.                                         |
| `just lint`       | Run Clippy with all targets and all features; warnings are errors. |
| `just lint-min`   | Run Clippy with default features disabled; warnings are errors.    |
| `just build`      | Build the release CLI with all features.                           |
| `just build-min`  | Build the release CLI without default features.                    |
| `just build-wasm` | Build `uuidx-wasm` for `wasm32-unknown-unknown`.                   |
| `just build-wasm-min` | Build target bindings without default features.                 |
| `just build-ffi`  | Build the release C ABI library.                                   |
| `just doc`        | Build workspace API documentation without dependencies.           |
| `just smoke`      | Run representative generation and JSON inspection commands.       |
| `just check`      | Run formatting, tests, and both lint matrices.                     |
| `just ci`         | Run `check` plus the CLI, WebAssembly, and FFI builds.             |

Coverage is optional:

```console
cargo install cargo-llvm-cov
just coverage
just coverage-lcov
```

Both recipes merge the all-features and no-default-features host test profiles
and enforce workspace floors of 97% line coverage, 96% function coverage, and
96% region coverage. These reports do not include doctests; WebAssembly export
behavior is checked separately under Node by `just test-wasm-target`.

## Automation and releases

GitHub Actions runs the main CI workflow for pushes and pull requests targeting
`main`, as well as manual runs. It checks formatting and Clippy, tests all
features and no-default-features on Linux, Windows, and macOS, verifies the
minimum Rust `1.88.0` toolchain, then builds the release CLI and runs smoke
checks. It also compiles `uuidx-wasm` for `wasm32-unknown-unknown`. The workflow
uses stable Rust for the normal matrix.

Additional repository automation includes:

- `coverage.yml` enforces the workspace coverage floors and uploads the merged
  feature-matrix LCOV report on pushes and pull requests.
- `codeql.yml` runs CodeQL security and quality analysis for Rust.
- `dependency-review.yml` checks pull request dependency changes.
- `scorecard.yml` runs OpenSSF supply-chain checks on the main branch and on a
  schedule.
- `dependabot.yml` checks Cargo dependencies weekly.
- `release.yml` accepts `vMAJOR.MINOR.PATCH` tags with optional `-beta[.N]` or
  `-rc[.N]` suffixes, verifies the tag against the workspace version, builds
  archives for Linux, Windows, and Intel and ARM macOS targets, generates
  provenance attestations, and publishes a GitHub release. Stable `v1.x.x`
  releases publish all four workspace crates to crates.io in dependency order;
  `0.x.x`, prerelease, and later major versions remain GitHub-only.

### Making changes

Keep changes within the existing module boundaries. New UUID behavior belongs
in the smallest relevant `uuidx-core` module and should receive a focused core
test before adapter wiring is added. CLI changes should keep parsing, input,
commands, rendering, and serialization in their current layers. Changes to JSON
fields require updates to [`docs/JSON.md`](docs/JSON.md), serialization tests,
and user-facing examples when applicable. WebAssembly and FFI changes belong
in their adapter crates and must keep language-specific types, errors, and
memory management outside `uuidx-core`.

See [`CONTRIBUTION.md`](CONTRIBUTION.md) for the complete contributor workflow,
testing expectations, and pull request checklist.
