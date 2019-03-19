#!/usr/bin/env bash
set -ex

cargo build --verbose
cargo test --verbose
