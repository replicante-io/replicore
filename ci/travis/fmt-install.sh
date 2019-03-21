#!/usr/bin/env bash
set -ex

if [ "$TRAVIS_RUST_VERSION" == "nightly" ]; then
  rustup component add rustfmt-preview
else 
  rustup component add rustfmt
fi
