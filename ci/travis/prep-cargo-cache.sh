#!/bin/bash
#
# This script "prunes" a cargo target directory based on a heuristic to preserve
# the cache unchanged across builds and limit the need to recomiple crates or
# re-upload the cache directory.
#
# Based on code and ideas from https://github.com/rust-lang/cargo/issues/5885
#
# The idea is to delete all crates from the project while preserving all extenral dependencies.
# This means that all replicante crates are re-built, even if they are not changed,
# but all external dependencies can be reused and the cache not uploaded every build.
#
# No attempts to deal with stale duplicate versions are made here (cargo updates).
#
set -ex

file_prefix=repli
target_path=${CARGO_TARGET_DIR-target}

# Prune the target directory to only keep dependencies.
if [ -e "${target_path}/debug" ]; then
  find "${target_path}/debug" -maxdepth 1 -type f -delete
  find "${target_path}/debug/.fingerprint" -name "${file_prefix}*" -exec rm -rf '{}' +
  find "${target_path}/debug/build" -name "${file_prefix}*" -exec rm -rf '{}' +
  find "${target_path}/debug/deps" -name "${file_prefix}*" -exec rm -rf '{}' +
  find "${target_path}/debug/deps" -name "lib${file_prefix}*" -exec rm -rf '{}' +
fi
rm -rf "${target_path}/debug/incremental"
rm -rf "${target_path}/.rustc_info.json"

# Prune the registries indexes since they change often.
rm -rf $HOME/.cargo/registry/index

# Prune the security advisory database since it changes often and we want it to.
rm -rf $HOME/.cargo/advisory-db
