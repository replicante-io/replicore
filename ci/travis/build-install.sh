#!/usr/bin/env bash
set -ex

if [ "$TRAVIS_RUST_VERSION" == "nightly" ]; then
  rustup component add clippy-preview
  rustup component add rustfmt-preview
else 
  rustup component add clippy
  rustup component add rustfmt
fi
