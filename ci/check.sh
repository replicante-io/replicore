#!/bin/bash -ex
#
#  Run checks against stable: ci/check.sh
#  Run checks against nightly: USE_NIGHTLY=yes rustup run nightly ci/check.sh
#  Run checks with coverage: USE_NIGHTLY=yes COVERAGE=yes rustup run nightly ci/check.sh
#
cargo test
[ "${USE_NIGHTLY}" == "yes" ] && cargo clippy
cargo outdated --root-deps-only

# Coverage tests are slow to build, only run when requested.
[ "${USE_NIGHTLY}" == "yes" ] && [ "${COVERAGE}" == "yes" ] cargo kcov --all
