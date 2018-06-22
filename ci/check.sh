#!/bin/bash -ex
#
#  Run checks against stable: ci/check.sh
#  Run checks against nightly: USE_NIGHTLY=yes rustup run nightly ci/check.sh
#  Run checks with coverage: USE_NIGHTLY=yes COVERAGE=yes rustup run nightly ci/check.sh
#
cargo clean
cargo test
if [ "${USE_NIGHTLY}" == "yes" ]; then
  cargo clippy
fi
cargo outdated --root-deps-only

# Coverage tests are slow to build, only run when requested.
if [ "${USE_NIGHTLY}" == "yes" -a "${COVERAGE}" == "yes" ]; then
  cargo kcov --all --verbose
fi
