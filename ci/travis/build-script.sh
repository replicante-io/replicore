#!/usr/bin/env bash
set -ex

cargo build
cargo test

# lint `clippy::cyclomatic_complexity` has been renamed to `clippy::cognitive_complexity`
# but clippy for rust 1.34 does not allow the new name yet.
# Allow lint renames until the minimum supported rust version is 1.35 and then patch mongodb:
# ```
#  sed -i 's/cyclomatic-complexity-threshold/cognitive-complexity-threshold/' \
#    ~/.cargo/registry/src/github.com-1ecc6299db9ec823/mongodb-0.3.12/clippy.toml
# ```
cargo clippy -- -D warnings -A renamed-and-removed-lints

# Run cargo fmt in verbose mode to see what the editions are.
cargo fmt --verbose -- --check
