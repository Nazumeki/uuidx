# uuidx

[English](README.md) · [简体中文](README.zh-CN.md)

[![CI](https://img.shields.io/github/actions/workflow/status/Nazumeki/uuidx/ci.yml?style=for-the-badge&logo=githubactions&logoColor=white&label=CI)](https://github.com/Nazumeki/uuidx/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/Nazumeki/uuidx?style=for-the-badge&logo=github&logoColor=white&label=release)](https://github.com/Nazumeki/uuidx/releases/latest)
[![crates.io](https://img.shields.io/crates/v/uuidx-cli?style=for-the-badge&logo=rust&logoColor=white&label=crates.io)](https://crates.io/crates/uuidx-cli)

[![Rust](https://img.shields.io/badge/Rust-1.88%2B-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Coverage](https://img.shields.io/badge/coverage-%E2%89%A597%25-brightgreen?style=for-the-badge)](https://github.com/Nazumeki/uuidx/actions/workflows/ci.yml)
[![License](https://img.shields.io/github/license/Nazumeki/uuidx?style=for-the-badge&color=blue&label=license)](LICENSE)

[![Stars](https://img.shields.io/github/stars/Nazumeki/uuidx?style=for-the-badge&logo=github&logoColor=white&label=stars)](https://github.com/Nazumeki/uuidx/stargazers)
[![Forks](https://img.shields.io/github/forks/Nazumeki/uuidx?style=for-the-badge&logo=github&logoColor=white&label=forks)](https://github.com/Nazumeki/uuidx/forks)
[![Commits](https://img.shields.io/github/commit-activity/t/Nazumeki/uuidx?style=for-the-badge&logo=github&logoColor=white&label=commits)](https://github.com/Nazumeki/uuidx/commits)
[![Issues](https://img.shields.io/github/issues/Nazumeki/uuidx?style=for-the-badge&logo=github&logoColor=white&label=issues)](https://github.com/Nazumeki/uuidx/issues)
[![PRs](https://img.shields.io/github/issues-pr/Nazumeki/uuidx?style=for-the-badge&logo=github&logoColor=white&label=PRs)](https://github.com/Nazumeki/uuidx/pulls)

`uuidx` is a Rust workspace for generating, inspecting, validating, and
formatting UUIDs. One domain crate powers the `uuidx` CLI, Rust applications,
JavaScript through WebAssembly, and native applications through a stable C ABI.

## Why uuidx

- Generate UUID v3, v4, v5, v6, v7, and v8 with version-specific options.
- Inspect UUID versions, variants, timestamps, node metadata, payloads, and bit
  layouts, including legacy, reserved, and unknown-version values.
- Validate and convert batches from arguments, files, or newline-delimited
  stdin.
- Emit terminal-friendly output, pipeline-safe plain values, or JSONL for
  automation.
- Automatically inspect UUIDs, standard 21-character NanoIDs, original Twitter
  Snowflakes, and (with `ulid-inspect`) ULIDs.
- Reuse the same policy from Rust, JavaScript, or C without adapter crates
  depending on one another.

## Install

Install the published CLI from crates.io:

```console
cargo install uuidx-cli --locked
uuidx --version
```

For a manual release-archive install, verification, and `PATH` setup, see
[`docs/INSTALLATION.md`](docs/INSTALLATION.md). For a checkout-based build,
custom features or targets, or library artifacts, see
[`docs/SOURCEBUILD.md`](docs/SOURCEBUILD.md).

## Quick start

Generate UUID v7 (the default):

```console
uuidx generate
uuidx generate v7 --output plain
```

Generate five UUID v4 values, one per line:

```console
uuidx generate v4 --count 5 --output plain
```

Generate a deterministic UUID v5:

```console
uuidx generate v5 --namespace dns --name example.com --output plain
```

Inspect a UUID as JSON, including timestamp and bit metadata:

```console
uuidx inspect 018f2c0b-6c5b-7d2e-8f4a-123456789abc --output json --layout
```

Validate a newline-delimited stream. Invalid records are reported on stderr and
return exit status `1`:

```console
printf '%s\n' 018f2c0b-6c5b-7d2e-8f4a-123456789abc not-a-uuid | uuidx validate --output plain
```

Convert to the 32-character simple form:

```console
uuidx convert 018f2c0b-6c5b-7d2e-8f4a-123456789abc --to simple --output plain
```

## UUID support policy

Inspection, validation, and conversion accept syntactically valid UUIDs even
when a version is not a generation target. Generation intentionally supports
only v3 through v8.

| Family                    | Inspect | Validate | Convert | Generate |
| ------------------------- | :-----: | :------: | :-----: | :------: |
| v1, v2                    |   Yes   |   Yes    |   Yes   |    No    |
| v3, v4, v5, v6, v7, v8    |   Yes   |   Yes    |   Yes   |   Yes    |
| Nil, Max, unknown version |   Yes   |   Yes    |   Yes   |    No    |

`v3` and `v5` require a namespace and name. `v6` and `v7` accept RFC 3339 or
Unix-millisecond timestamps. `v8` requires exactly 16 bytes of hexadecimal
application data. Legacy MD5/SHA-1 generation remains available for standards
compatibility and emits a warning.

## Command reference

```text
uuidx [OPTIONS] <COMMAND>
```

| Command    | Alias | Purpose                                           |
| ---------- | ----- | ------------------------------------------------- |
| `generate` | `g`   | Generate UUID v3-v8 values.                       |
| `inspect`  | `i`   | Inspect UUIDs and supported identifier families.  |
| `validate` | `v`   | Validate UUID syntax in batches.                  |
| `convert`  | `c`   | Convert canonical, simple, URN, and braced forms. |

Global output modes are `auto`, `pretty`, `plain`, and `json`. Use
`uuidx <command> --help` for the parser-generated option reference. The JSONL
contract for automation is documented in [`docs/JSON.md`](docs/JSON.md).

## Integrations

### Rust

`uuidx-core` has no default features and exposes parsing, generation, formatting,
validation, and inspection without CLI or FFI dependencies:

```toml
[dependencies]
uuidx-core = "0.1"
```

```rust
use uuidx_core::{generate_uuid, GeneratableUuidVersion, GenerationOptions};

let value = generate_uuid(&GenerationOptions::new(GeneratableUuidVersion::V7))?;
println!("{value}");
```

Enable `ulid-inspect` when a Rust application needs read-only ULID inspection.

### WebAssembly

`uuidx-wasm` exports `generateUuid`, `validateUuid`, `formatUuid`,
`inspectUuid`, `inspectIdentifier`, `inspectUlid`, `inspectNanoid`, and
`inspectSnowflake` through `wasm-bindgen`. Build the supported target with:

```console
rustup target add wasm32-unknown-unknown
just build-wasm
```

### C ABI

`uuidx-ffi` builds static and dynamic libraries and publishes its header at
[`crates/uuidx-ffi/include/uuidx.h`](crates/uuidx-ffi/include/uuidx.h):

```console
just build-ffi
```

## Architecture

```text
uuidx-cli  ---+
uuidx-wasm ---+--> uuidx-core --> uuid
uuidx-ffi  ---+
```

`uuidx-core` owns domain behavior and public policy. The CLI owns input and
presentation; WebAssembly owns JavaScript conversion; FFI owns the C ABI and
memory boundary. The crate ownership, dependency direction, and extension rules
are documented in [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

## Contributing

Changes to crate boundaries should follow [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).
Changes to JSON output should follow [`docs/JSON.md`](docs/JSON.md). The
contributor workflow, testing expectations, and pull request checklist are in
[`CONTRIBUTION.md`](CONTRIBUTION.md).

## License

`uuidx` is available under the [MIT License](LICENSE).
