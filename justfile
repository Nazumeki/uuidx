default: check

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

test:
    cargo test --workspace --all-features

test-min:
    cargo test --workspace --no-default-features

test-core:
    cargo test -p uuidx-core --all-features

test-cli:
    cargo test -p uuidx-cli --all-features

test-wasm:
    cargo test -p uuidx-wasm --all-features

test-wasm-target:
    WASM_BINDGEN_BENCH_RESULT=target/wbg_benchmark.json cargo test --locked -p uuidx-wasm --target wasm32-unknown-unknown --all-features
    WASM_BINDGEN_BENCH_RESULT=target/wbg_benchmark.json cargo test --locked -p uuidx-wasm --target wasm32-unknown-unknown --no-default-features

test-ffi:
    cargo test -p uuidx-ffi --all-features

test-all: test test-min

lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

lint-min:
    cargo clippy --workspace --all-targets --no-default-features -- -D warnings

build:
    cargo build --release -p uuidx-cli --all-features

build-min:
    cargo build --release -p uuidx-cli --no-default-features

build-wasm:
    cargo build --locked -p uuidx-wasm --target wasm32-unknown-unknown --all-features

build-wasm-min:
    cargo build --locked -p uuidx-wasm --target wasm32-unknown-unknown --no-default-features

build-ffi:
    cargo build --release -p uuidx-ffi --all-features

doc:
    cargo doc --workspace --all-features --no-deps

coverage:
    cargo llvm-cov clean --workspace
    cargo llvm-cov --workspace --all-features --no-report
    cargo llvm-cov --workspace --no-default-features --no-report
    cargo llvm-cov report --ignore-filename-regex 'uuidx-wasm[/\\]src[/\\]lib\.rs$' --fail-under-lines 97 --fail-under-functions 96 --fail-under-regions 96 --html

coverage-lcov:
    cargo llvm-cov clean --workspace
    cargo llvm-cov --workspace --all-features --no-report
    cargo llvm-cov --workspace --no-default-features --no-report
    cargo llvm-cov report --ignore-filename-regex 'uuidx-wasm[/\\]src[/\\]lib\.rs$' --fail-under-lines 97 --fail-under-functions 96 --fail-under-regions 96 --lcov --output-path target/coverage/lcov.info

smoke:
    cargo run --quiet -p uuidx-cli --all-features -- generate v4 --output plain
    cargo run --quiet -p uuidx-cli --all-features -- inspect 018f2c0b-6c5b-7d2e-8f4a-123456789abc --output json

# Requires cargo-fuzz (`cargo install cargo-fuzz`) and a nightly toolchain.
fuzz:
    cargo fuzz run parse_uuid

check: fmt-check test-all lint lint-min

ci: check test-wasm-target build build-min build-wasm build-wasm-min build-ffi
