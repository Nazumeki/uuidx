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

test-all: test test-min

lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

lint-min:
    cargo clippy --workspace --all-targets --no-default-features -- -D warnings

build:
    cargo build --release -p uuidx-cli --all-features

build-min:
    cargo build --release -p uuidx-cli --no-default-features

doc:
    cargo doc --workspace --all-features --no-deps

coverage:
    cargo llvm-cov --workspace --all-features --html

coverage-lcov:
    cargo llvm-cov --workspace --all-features --lcov --output-path target/coverage/lcov.info

smoke:
    cargo run --quiet -p uuidx-cli --all-features -- generate v4 --output plain
    cargo run --quiet -p uuidx-cli --all-features -- inspect 018f2c0b-6c5b-7d2e-8f4a-123456789abc --output json

check: fmt-check test-all lint lint-min

ci: check build build-min
