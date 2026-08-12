# Contributing to uuidx

This document describes the development workflow and design constraints for
changes to `uuidx`. The project is a small Rust workspace, but its CLI output,
core API, feature matrix, and JSON records are public contracts. Contributions
should keep those boundaries explicit and leave the repository in a verified,
usable state.

## Project principles

The repository follows these principles:

- Keep the smallest implementation that completely satisfies the current
  requirement.
- Grow functionality in working layers: core behavior, focused tests, adapter
  wiring, presentation, and documentation.
- Keep all adapters dependent on `uuidx-core`, never on each other.
- Put domain rules in `uuidx-core`; keep terminal and process concerns in
  `uuidx-cli`, JavaScript conversion in `uuidx-wasm`, and C ABI ownership in
  `uuidx-ffi`.
- Prefer existing dependencies and local patterns over new abstractions or
  duplicate implementations.
- Do not add compatibility layers for obsolete paths. If a behavior is no
  longer part of the current contract, remove it and update its documentation
  and tests.
- Treat sensitive UUID metadata, especially time-based node identifiers, as
  data that may require deliberate redaction.

The current generation policy is intentional: v3, v4, v5, v6, v7, and v8 are
generation targets; v1 and v2 remain available for parsing, inspection,
validation, and conversion. ULID, NanoID, and Snowflake support is
inspection-only; only ULID inspection is optional.
Changes that broaden this policy need a clear design decision and matching
tests rather than a fallback path.

## Before opening an issue

Search existing issues and documentation first. For a reproducible bug report,
include:

- the `uuidx --version` output;
- operating system and Rust toolchain information;
- the exact command or library call;
- a minimal input sample with secrets and sensitive identifiers removed;
- selected output mode and complete exit status;
- expected behavior, actual behavior, and reproduction steps.

Do not include credentials, private UUID data, unredacted hardware-derived node
IDs, or other sensitive values in a public issue. For a suspected security
issue, use a private maintainer channel instead of publishing an exploit or
proof of concept in a public issue.

## Development setup

The minimum supported Rust version is `1.88`. The workspace uses edition 2024.
Install Rust and Cargo through [rustup](https://rustup.rs/), then verify the
checkout:

```console
rustc --version
cargo --version
```

Install [Just](https://github.com/casey/just) for the repository's canonical
recipes. The project does not require a global toolchain beyond Rust, Just, and
the optional `cargo-llvm-cov` coverage tool. Install the WebAssembly target
before running the full local CI recipe:

```console
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.127 --locked
```

The workspace contains:

```text
crates/uuidx-core/       UUID domain library and core integration tests
crates/uuidx-cli/        CLI binary, command tests, and presentation layers
crates/uuidx-wasm/       WebAssembly exports and JavaScript conversion
crates/uuidx-ffi/        C ABI library and public header
docs/                    User installation/build guides and developer contracts
justfile                Shared local and CI commands
```

Build and run the CLI directly from the checkout:

```console
cargo build --package uuidx-cli
cargo run --package uuidx-cli -- generate v7
```

The CLI enables `ulid-inspect` by default. The no-default-feature build is a
supported configuration and must continue to compile and test:

```console
cargo test --workspace --no-default-features
cargo build --release --package uuidx-cli --no-default-features
```

## Standard change workflow

1. Start from an up-to-date branch based on `main`.
2. Read the relevant module, tests, and documentation before changing the
   contract.
3. Make a focused change in the smallest responsible crate and module.
4. Add or update tests at the same layer as the behavior being changed.
5. Update CLI help, README examples, architecture notes, or the JSON contract
   when the user-facing behavior changes.
6. Run the appropriate focused checks while iterating, then run `just ci`
   before opening a pull request.
7. Review the final diff for unrelated changes, generated files, accidental
   ANSI output, sensitive test data, and stale documentation.

Keep commits and pull requests focused. A change that combines a core policy
decision, a CLI redesign, and unrelated formatting is difficult to review and
hard to validate.

## Where changes belong

### Core behavior

`crates/uuidx-core/src/` is organized by responsibility:

| Area | Responsibility |
| --- | --- |
| `types.rs` | Public UUID types, version enums, options, and variants. |
| `error.rs` | Public domain errors for parsing, payloads, and generation options. |
| `parse.rs` | UUID text parsing. |
| `format.rs` | UUID output formats and fixed-size hexadecimal payload parsing. |
| `generate/` | Version-specific v3-v8 generation. |
| `inspect/` | Version classification, variants, metadata, and bit layouts. |
| `inspect/identifier.rs` | Automatic identifier-family recognition. |
| `inspect/nanoid.rs` | Standard 21-character NanoID inspection. |
| `inspect/snowflake.rs` | Original Twitter Snowflake inspection. |
| `inspect/ulid.rs` | Optional read-only ULID inspection. |

When adding or changing a UUID rule:

1. Put the construction or inspection logic in the smallest relevant core
   module.
2. Add a public type or error only when it is part of the caller-facing
   contract.
3. Preserve the distinction between `GeneratableUuidVersion` and
   `InspectableUuidVersion`.
4. Add a focused core integration test before adding CLI behavior.
5. Check both all-feature and no-default-feature builds when the change touches
   feature-gated code.

The public `uuidx_core::Uuid` is a wrapper around the `uuid` crate's value. The
underlying constructors stay private to the core crate so callers cannot bypass
the generation policy through a re-export. Do not reintroduce that escape hatch
for convenience.

### CLI behavior

`crates/uuidx-cli/src/` has separate boundaries:

| Area | Responsibility |
| --- | --- |
| `cli/` | Clap arguments, subcommands, aliases, and value parsing. |
| `commands/` | Generate, inspect, validate, and convert workflows. |
| `input.rs` | Positional values, files, stdin, blank lines, and fail-fast behavior. |
| `output/` | Render mode selection and stream-safe output. |
| `output/pretty/` | Human-readable sections, themes, and optional ANSI styling. |
| `output/json.rs` | Newline-delimited JSON records and serialization. |
| `errors.rs` | Process-level error mapping and exit statuses. |
| `app.rs` | Thin dispatch and flush boundary. |

Commands should produce domain values or domain errors and pass them to the
output layer. Do not put terminal styling in `uuidx-core`, serialize JSON in a
generation module, or make a renderer responsible for UUID bit rules.

### JSON and machine output

The output contract is part of the public interface:

- `plain` output must remain suitable for shell pipelines and must not contain
  ANSI escape sequences.
- `json` output is newline-delimited JSON, with one record per generated or
  processed input value.
- JSON records use `schema_version`, currently `1`, and omit fields that have
  no value.
- Record-level invalid data is represented as an error record in JSON mode and
  still produces process status `1`.
- Top-level usage and I/O failures remain process errors with status `2` or
  `3`, rather than being confused with an invalid record.
- `inspect --redact-sensitive` may remove a node ID value but must retain its
  node classification.

When changing JSON fields or metadata:

1. Update the serializer and its unit tests.
2. Update [`docs/JSON.md`](docs/JSON.md), including the field table and an
   example if the shape changes.
3. Update CLI integration tests for success, error, and feature-gated records.
4. Update [`README.md`](README.md) when the command contract or examples are
   affected.
5. Prefer additive fields and tolerant consumers, but do not add compatibility
   serializers or duplicate schemas for obsolete behavior.

### WebAssembly and C ABI adapters

`uuidx-wasm` and `uuidx-ffi` translate the core API at language boundaries.
They must not duplicate UUID parsing, generation, formatting, inspection, or
policy logic from `uuidx-core`.

WebAssembly changes should use JavaScript-native values, preserve structured
error codes, and compile for `wasm32-unknown-unknown`. FFI changes must keep the
public header synchronized with the Rust exports and make allocation ownership
explicit. New exported functions require focused adapter tests and public usage
documentation.

## Testing and verification

The root `justfile` is the canonical test and verification interface.

| Recipe | What it checks |
| --- | --- |
| `just fmt` | Formats all Rust code. |
| `just fmt-check` | Fails when formatting is needed. |
| `just test` | Workspace tests with all features. |
| `just test-min` | Workspace tests with default features disabled. |
| `just test-core` | Core crate tests with all features. |
| `just test-cli` | CLI crate tests with all features. |
| `just test-wasm` | WebAssembly adapter host tests with all features. |
| `just test-wasm-target` | WebAssembly exports under Node in both feature modes. |
| `just test-ffi` | C ABI adapter tests with all features. |
| `just test-all` | Both feature matrices. |
| `just lint` | Clippy for all workspace targets and features with `-D warnings`. |
| `just lint-min` | Clippy with default features disabled and `-D warnings`. |
| `just build` | Release CLI build with all features. |
| `just build-min` | Release CLI build without default features. |
| `just build-wasm` | `uuidx-wasm` build for `wasm32-unknown-unknown`. |
| `just build-wasm-min` | Target build without default features. |
| `just build-ffi` | Release C ABI library build. |
| `just doc` | Workspace API documentation without dependency docs. |
| `just smoke` | Representative generation and JSON inspection commands. |
| `just check` | Formatting, tests, and both lint matrices. |
| `just ci` | Run `check` plus the CLI, WebAssembly, and FFI builds before review. |

During development, use the narrowest useful check first:

```console
just fmt
just test-core
just test-cli
just lint
```

Before submitting a pull request, run the complete matrix:

```console
just ci
just smoke
```

Coverage reports are optional and require `cargo-llvm-cov`:

```console
cargo install cargo-llvm-cov
just coverage
just coverage-lcov
```

The coverage recipes merge the all-features and no-default-features host test
profiles. They enforce workspace floors of 97% for lines and 96% for functions
and regions. The report excludes doctests and the target-only
`uuidx-wasm/src/lib.rs` export boundary; `just test-wasm-target` separately
executes the WebAssembly exports under Node.

### Test expectations by change type

- Core parsing, formatting, generation, or inspection changes need focused
  core tests and regression coverage for invalid options or values.
- CLI argument changes need parser or command integration tests, including
  aliases, defaults, and invalid combinations where applicable.
- Input changes need tests for positional values, files, stdin, blank lines,
  missing input, and `--fail-fast` behavior when those paths are affected.
- Output changes need coverage for plain, pretty, and JSON modes. Machine
  output must be checked for the absence of ANSI escape sequences.
- JSON changes need serialization assertions for successful records, error
  records, metadata families, and redaction when relevant.
- Feature-gated changes need both `just test` and `just test-min`, plus the
  corresponding lint and build checks.
- WebAssembly changes need host tests, `just test-wasm-target`, and
  `just build-wasm`; FFI changes need adapter tests, `just build-ffi`, and a
  header/API consistency review.
- Documentation-only changes should still be checked for stale commands,
  broken relative links, and examples that contradict `--help` or tests.

## CLI smoke checks

These are the same representative operations used by the `smoke` recipe:

```console
cargo run --quiet --package uuidx-cli --all-features -- \
  generate v4 --output plain
cargo run --quiet --package uuidx-cli --all-features -- \
  inspect 018f2c0b-6c5b-7d2e-8f4a-123456789abc --output json
```

When testing a change that affects batch behavior, also exercise a file and a
pipe, and verify both output streams and the exit status. For time-based UUIDs,
use an explicit timestamp and node where deterministic assertions are needed.
For v8, use a fixed 32-character hexadecimal payload.

## Pull request checklist

Before requesting review, confirm:

- [ ] The change is scoped to the current requirement and does not add a
      speculative abstraction or compatibility path.
- [ ] Core rules remain in `uuidx-core`; CLI, JavaScript, and C ABI concerns
      remain in their adapter crates.
- [ ] Tests cover the changed behavior and relevant error paths.
- [ ] Both default-feature and no-default-feature checks pass when applicable.
- [ ] Plain and JSON output remain machine-safe when affected.
- [ ] JSON schema documentation and tests are updated when fields change.
- [ ] README and architecture documentation match the implementation.
- [ ] `just ci` and the relevant smoke checks pass locally.
- [ ] No generated build artifacts, private data, or unrelated formatting
      changes are included.
- [ ] The pull request explains the motivation, implementation boundary, tests
      run, and any user-visible behavior change.

The main GitHub Actions workflow additionally runs formatting, Clippy, and the
all-feature and minimal-feature test matrix on Linux, Windows, and macOS. It
also verifies the minimum Rust `1.88.0` toolchain before the release build and
smoke checks, and compiles `uuidx-wasm` for `wasm32-unknown-unknown`. Separate
workflows cover coverage, CodeQL, dependency review, supply-chain checks, and
tagged release packaging for all four crates.

## Review expectations

Reviews prioritize correctness and contract clarity:

1. UUID version and variant behavior must be defensible and covered by tests.
2. Unsupported generation must fail explicitly rather than silently falling
   back to another version or constructor.
3. Public output and exit statuses must remain predictable for pipelines.
4. Sensitive metadata must not be exposed accidentally in new output paths.
5. New dependencies and abstractions must reduce complexity enough to justify
   their maintenance cost.
6. Documentation must describe behavior that exists today, including limits and
   feature flags.
7. Language bindings must preserve the core policy and define their error and
   memory contracts precisely.

## License

Contributions should be compatible with the repository's [MIT License](LICENSE).
