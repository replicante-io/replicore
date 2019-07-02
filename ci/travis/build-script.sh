#!/usr/bin/env bash
set -ex

cargo build
cargo test

# lint `clippy::cyclomatic_complexity` has been renamed to `clippy::cognitive_complexity`
# in clippy for rust 1.34.0.
# Supporting both 1.34 and 1.35 is a pain/impossible so upgrade to 1.35.
#sed -i 's/cyclomatic-complexity-threshold/cognitive-complexity-threshold/' \
#  ~/.cargo/registry/src/github.com-1ecc6299db9ec823/mongodb-0.3.12/clippy.toml
cargo clippy -- -D warnings

# Run cargo fmt in verbose mode to see what the editions are.
cargo fmt --verbose -- --check
