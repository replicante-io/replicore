#!/bin/env bash
set -e

# Confguration variables.
VERSON=""

# Parse CLI args.
usage() {
  echo 'Usage: ci/release/version-docs.sh vX.Y.Z'
}

while [[ $# -ne 0 ]]; do
  arg=$1
  shift

  case "${arg}" in
    v[0-9].[0-9].[0-9]) VERSON=$arg;;
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

if [[ -z ${VERSON} ]]; then
  echo 'Need a VERSON to release'
  usage
  exit 1
fi

# Version docs
version_project() {
  echo "==> Versioning $1 ..."
  pushd $1 > /dev/null
  npm run version "${VERSON#v}"
  popd > /dev/null
}

version_project 'docs/manual/website'
version_project 'docs/notes/website'
version_project 'docs/specs/website'
