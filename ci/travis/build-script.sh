#!/usr/bin/env bash
set -ex

cargo build
cargo test
cargo clippy -- -D warnings

# Run cargo fmt in verbose mode to see what the editions are.
cargo fmt --verbose -- --check
