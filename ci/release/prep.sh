#!/bin/env bash
set -e

# Confguration variables.
sudo=""
version=""

# Parse CLI args.
usage() {
  echo 'Usage: ci/release/prep.sh [OPTIONS] VERSION'
  echo
  echo 'VERSION is the version to release and is in the format vX.Y.Z'
  echo
  echo 'Available options:'
  echo '  --sudo    Use sudo for docker commands'
}

while [[ $# -ne 0 ]]; do
  arg=$1
  shift

  case "${arg}" in
    v[0-9].[0-9].[0-9]) version=$arg;;
    --sudo) sudo="--sudo";;

    --help|help|-h|h)
      usage
      exit 0
      ;;
    *)
      echo "Unrecognised argument ${arg}"
      usage
      exit 1
  esac
done

if [[ -z ${version} ]]; then
  echo 'Need a version to release'
  usage
  exit 1
fi

# Ensure dependencies are up to date, tests are clear, and Cargo.lock is fresh
echo '==> Updating deps and running tests ...'
cargo clean
cargo update
ci/travis/build-script.sh

echo '==> Checking docker images build successfully ...'
ci/release/docker.sh check "${version}" ${sudo}
