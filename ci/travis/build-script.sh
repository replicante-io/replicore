#!/usr/bin/env bash
set -ex

cargo build --verbose
cargo test --verbose
cargo clippy --verbose # TODO: -- -D warnings
# Code format is optional until we can make it work.
cargo fmt --verbose -- --check || true

# Until all crates can be fully checked re-check the ones
# we made compatible to prevent slips.
cargo clippy --verbose -p replicante_agent_client -- -D warnings
cargo clippy --verbose -p replicante_agent_discovery -- -D warnings
cargo clippy --verbose -p replicante_coordinator -- -D warnings
cargo clippy --verbose -p replicante_data_aggregator -- -D warnings
cargo clippy --verbose -p replicante_data_models -- -D warnings
cargo clippy --verbose -p replicante_data_store -- -D warnings
cargo fmt --verbose -preplicante_agent_client -- --check
cargo fmt --verbose -preplicante_agent_discovery -- --check
cargo fmt --verbose -preplicante_coordinator -- --check
cargo fmt --verbose -preplicante_data_aggregator -- --check
cargo fmt --verbose -preplicante_data_models -- --check
cargo fmt --verbose -preplicante_data_store -- --check
