#!/bin/bash
set -e

# Check for required arguments
if [ $# -ne 2 ]; then
  echo "Usage: ci/test-and-lint-worksapce.sh NAME MANIFEST-PATH" >&2
  exit 1
fi
NAME="$1"
MANIFEST="$2"

# GitHub Actions log group support, when running in CI only.
log_group() {
  if [ -n "${CI}" ]; then
    echo "::group::$1"
  else
    echo "$1"
  fi
}
log_group_end() {
  if [ -n "${CI}" ]; then
    echo "::endgroup::"
  fi
}

# Build, test, clippy, format stages
log_group "Build ${NAME} packages"
cargo build --manifest-path "${MANIFEST}"
log_group_end

log_group "Run ${NAME} tests"
cargo test --manifest-path "${MANIFEST}"
log_group_end

log_group "Run ${NAME} clippy"
cargo clippy --manifest-path "${MANIFEST}" -- -D warnings
log_group_end

log_group "Check ${NAME} formatting"
# Cargo fmt behaves oddly with "Cargo.toml" for the manifest path.
# Pass the manifest path only if not the root crate.
# Issue: https://github.com/rust-lang/rustfmt/issues/4432
if [ "${MANIFEST}" == "Cargo.toml" ]; then
  cargo fmt --verbose -- --check
else
  cargo fmt --manifest-path "${MANIFEST}" --verbose -- --check
fi
log_group_end
