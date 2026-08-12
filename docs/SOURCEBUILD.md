# Build and install from source

This guide is for users who need a custom `uuidx` build: a local checkout, a
different feature set, a non-default target, or a library artifact. For a
published CLI binary or a crates.io install, use [`INSTALLATION.md`](INSTALLATION.md).

## Requirements

- Rust `1.88` or newer with Cargo.
- A C toolchain when building or linking `uuidx-ffi`.
- The `wasm32-unknown-unknown` Rust target when building `uuidx-wasm`.
- `wasm-bindgen-cli` when turning the WebAssembly artifact into a JavaScript
  package.

The `just` command is optional. Every build below uses Cargo directly.

## Build the CLI

Clone the repository and build an optimized CLI with all default features:

```console
git clone https://github.com/Nazumeki/uuidx.git
cd uuidx
cargo build --locked --release --package uuidx-cli --all-features
```

The result is `target/release/uuidx` on Unix-like systems and
`target\\release\\uuidx.exe` on Windows. Run the locally built binary without
installing it:

```console
cargo run --locked --package uuidx-cli -- generate v7
```

Install the local checkout into Cargo's binary directory:

```console
cargo install --locked --path crates/uuidx-cli --all-features
uuidx --version
```

To install it into a directory you control, use Cargo's `--root` option and
then add its `bin` directory to `PATH`:

```console
cargo install --locked --path crates/uuidx-cli --all-features --root "$HOME/.local"
export PATH="$HOME/.local/bin:$PATH"
```

On Windows, use a PowerShell path for `--root` and add its `bin` directory to
the user `PATH` as described in [`INSTALLATION.md`](INSTALLATION.md).

## Select features

`ulid-inspect` is the only optional feature in the workspace:

| Package | Default | Effect |
| --- | --- | --- |
| `uuidx-core` | None | Adds read-only ULID inspection when enabled explicitly. |
| `uuidx-cli` | `ulid-inspect` | Enables ULID recognition in `uuidx inspect`. |
| `uuidx-wasm` | `ulid-inspect` | Exposes ULID inspection to JavaScript. |
| `uuidx-ffi` | None | No optional features. |

Build a smaller CLI without ULID inspection:

```console
cargo build --locked --release --package uuidx-cli --no-default-features
cargo install --locked --path crates/uuidx-cli --no-default-features
```

When using a package as a dependency, enable the feature in that package's
manifest instead of changing the workspace build:

```toml
[dependencies]
uuidx-core = { version = "0.1", features = ["ulid-inspect"] }
```

## Build for another target

Install the target with `rustup`, then pass it to Cargo:

```console
rustup target add x86_64-unknown-linux-gnu
cargo build --locked --release --package uuidx-cli --target x86_64-unknown-linux-gnu --all-features
```

The binary is written below
`target/x86_64-unknown-linux-gnu/release/`. A cross-compiled binary may need a
target-specific linker and system libraries; configure those through Cargo's
normal target configuration rather than copying host libraries into the build.

## Build the Rust library

`uuidx-core` is the reusable Rust domain library. Build and test it directly:

```console
cargo build --locked --release --package uuidx-core --all-features
cargo test --locked --package uuidx-core --all-features
```

For an application in a neighboring checkout, use a path dependency while
developing against the local source:

```toml
[dependencies]
uuidx-core = { path = "../uuidx/crates/uuidx-core" }
```

## Build WebAssembly bindings

Add the supported target and build `uuidx-wasm`:

```console
rustup target add wasm32-unknown-unknown
cargo build --locked --package uuidx-wasm --target wasm32-unknown-unknown --all-features
```

The Rust artifact is placed in
`target/wasm32-unknown-unknown/debug/` or `target/wasm32-unknown-unknown/release/`,
depending on whether `--release` was used. To generate JavaScript bindings,
install a matching `wasm-bindgen-cli` and pass the resulting `.wasm` file to
`wasm-bindgen`:

```console
cargo install wasm-bindgen-cli --version 0.2.127 --locked
wasm-bindgen target/wasm32-unknown-unknown/debug/uuidx_wasm.wasm --out-dir pkg --target web
```

Use `--no-default-features` on the Cargo build when ULID inspection should not
be included.

## Build the C ABI

Build the static and dynamic C libraries:

```console
cargo build --locked --release --package uuidx-ffi --all-features
```

The public header is
[`crates/uuidx-ffi/include/uuidx.h`](../crates/uuidx-ffi/include/uuidx.h).
Libraries are written to `target/release/` with platform-specific names and
extensions. Keep the header and library from the same source revision when
linking an application.

## Verify a custom build

Run a focused check after changing the build configuration:

```console
cargo test --locked --workspace --all-features
cargo test --locked --workspace --no-default-features
cargo run --locked --quiet --package uuidx-cli --all-features -- generate v4 --output plain
```

The root `justfile` provides equivalent combined recipes such as `just check`,
`just build-wasm`, and `just build-ffi`. Contributor workflow and review
requirements are documented in [`CONTRIBUTION.md`](../CONTRIBUTION.md).
