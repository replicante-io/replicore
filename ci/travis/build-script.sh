#!/usr/bin/env bash
set -ex

cargo build
cargo test

# mongodb crate has an invalid clippy config on newer clippy.
cargo clippy --all --exclude mongodb -- -D warnings

# Run cargo fmt in verbose mode to see what the editions are.
cargo fmt --verbose -- --check
