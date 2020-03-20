#!/usr/bin/env bash
set -ex

# Replicante Core workspace.
cargo build
cargo test
cargo clippy -- -D warnings
cargo fmt --verbose -- --check


# Replicante development tool CI.
cargo build --manifest-path devtools/replidev/Cargo.toml
cargo test --manifest-path devtools/replidev/Cargo.toml
cargo clippy --manifest-path devtools/replidev/Cargo.toml -- -D warnings
cargo fmt --manifest-path devtools/replidev/Cargo.toml --verbose -- --check
