# uuidx

[![GitHub Stars](https://www.shieldcn.dev/github/stars/Nazumeki/uuidx.svg?variant=secondary&size=sm)](https://github.com/Nazumeki/uuidx/stargazers)
[![GitHub Forks](https://www.shieldcn.dev/github/forks/Nazumeki/uuidx.svg?variant=secondary&size=sm)](https://github.com/Nazumeki/uuidx/forks)
[![License](https://www.shieldcn.dev/github/license/Nazumeki/uuidx.svg?variant=secondary&size=sm)](LICENSE)
[![Release](https://www.shieldcn.dev/github/release/Nazumeki/uuidx.svg?variant=secondary&size=sm)](https://github.com/Nazumeki/uuidx/releases/latest)
[![CI](https://www.shieldcn.dev/github/ci/Nazumeki/uuidx.svg?variant=secondary&size=sm)](https://github.com/Nazumeki/uuidx/actions/workflows/ci.yml)

[![Commits](https://www.shieldcn.dev/github/commits/Nazumeki/uuidx.svg?variant=secondary&size=sm)](https://github.com/Nazumeki/uuidx/commits)
[![Open issues](https://www.shieldcn.dev/github/open-issues/Nazumeki/uuidx.svg?variant=secondary&size=sm)](https://github.com/Nazumeki/uuidx/issues)
[![Open PRs](https://www.shieldcn.dev/github/open-prs/Nazumeki/uuidx.svg?variant=secondary&size=sm)](https://github.com/Nazumeki/uuidx/pulls)

[![Rust](https://img.shields.io/badge/Rust-1.88%2B-000000?logo=rust&logoColor=white&style=flat)](https://www.rust-lang.org/)
[![crates.io](https://img.shields.io/crates/v/uuidx-cli?logo=rust&logoColor=white=flat)](https://crates.io/crates/uuidx-cli)
[![coverage](https://img.shields.io/badge/test%20coverage-%E2%89%A597-2ea44f=flat)](https://github.com/Nazumeki/uuidx/actions/workflows/coverage.yml)

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

For a direct manual installation, use [`docs/INSTALLATION.md`](docs/INSTALLATION.md)
to download a release archive, verify it, and configure `PATH`. The same guide
covers installing from crates.io with Cargo and setting Cargo's binary path when
needed. For a checkout-based build, custom features or targets, or library
artifacts, see
[`docs/SOURCEBUILD.md`](docs/SOURCEBUILD.md).

Install the published CLI from crates.io:

```console
cargo install uuidx-cli --locked
uuidx --version
```

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
