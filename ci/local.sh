#!/bin/bash
set -e

for_version() {
  version="$1"
  full_mode=""
  if [ "${version}" == "stable" ]; then
    full_mode="--full"
  fi

  echo "Clean up workspaces for version ${version}"
  rustup run "${version}" cargo clean
  rustup run "${version}" cargo clean --manifest-path devtools/replidev/Cargo.toml

  echo "Run CI for version ${version}"
  rustup run "${version}" ci/check-workspace.sh ${full_mode} Core Cargo.toml
  rustup run "${version}" ci/check-workspace.sh ${full_mode} "Replicante Development Tool" \
    devtools/replidev/Cargo.toml
}

for_version "stable"
for_version "1.64.0"
for_version "nightly"
