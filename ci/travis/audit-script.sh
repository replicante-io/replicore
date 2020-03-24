#!/usr/bin/env bash
set -ex

# Audit main workspace.
cargo audit

# Audit replidev crate.
cd devtools/replidev
cargo audit
