#!/usr/bin/env sh
set -ex

cargo build --verbose
cargo test --verbose
