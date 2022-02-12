#!/bin/bash
set -e

for_version() {
  version="$1"

  echo "Clean up workspaces for version ${version}"
  rustup run "${version}" cargo clean
  rustup run "${version}" cargo clean --manifest-path devtools/replidev/Cargo.toml

  echo "Run CI for version ${version}"
  rustup run "${version}" ci/test-and-lint-workspace.sh Core Cargo.toml
  rustup run "${version}" ci/test-and-lint-workspace.sh "Replicante Development Tool" \
    devtools/replidev/Cargo.toml
}

for_version "stable"
for_version "1.56.0"
for_version "nightly"
