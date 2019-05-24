#!/usr/bin/env bash
set -ex

cargo build
cargo test

# mongodb crate has an invalid clippy config on newer clippy versions.
sed -i 's/cyclomatic-complexity-threshold/cognitive-complexity-threshold/' ~/.cargo/registry/src/github.com-1ecc6299db9ec823/mongodb-0.3.12/clippy.toml
cargo clippy -- -D warnings

# Run cargo fmt in verbose mode to see what the editions are.
cargo fmt --verbose -- --check
