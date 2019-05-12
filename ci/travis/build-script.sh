#!/usr/bin/env bash
set -ex

cargo build
cargo test
cargo clippy -- -D warnings
# Run cargo fmt in verbose mode to see what the editions are.
# Code format is optional until we can make it work.
cargo fmt --verbose -- --check || true

# Until all crates can be fully checked re-check the ones
# we made compatible to prevent slips.
cargo fmt --verbose -preplicante_agent_client -- --check
cargo fmt --verbose -preplicante_agent_discovery -- --check
cargo fmt --verbose -preplicante_coordinator -- --check
cargo fmt --verbose -preplicante_data_aggregator -- --check
cargo fmt --verbose -preplicante_data_fetcher -- --check
cargo fmt --verbose -preplicante_data_models -- --check
cargo fmt --verbose -preplicante_data_store -- --check
cargo fmt --verbose -preplicante_streams_events -- --check
cargo fmt --verbose -preplicante_tasks -- --check
cargo fmt --verbose -preplicante -- --check
