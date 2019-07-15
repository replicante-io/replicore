#!/bin/env bash
set -e

# Confguration variables.
sudo=""
version=""

# Parse CLI args.
usage() {
  echo 'Usage: ci/release/artefacts.sh [OPTIONS] VERSION'
  echo
  echo 'VERSION is the version for the new docs and is in the format vX.Y.Z'
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

# Release docker images
echo '==> Building and pushing images'
ci/release/docker.sh release "${version}" ${sudo} --verbose
ci/release/gh-pre-built.sh --clean ${sudo} "${version}"
