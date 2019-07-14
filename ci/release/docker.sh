#!/bin/env bash
set -e

# Confguration variables.
COMMAND=""
DOCKE_REPO="replicanteio"
VERSON=""
VERBOSE="no"

# Parse CLI args.
usage() {
  echo 'Usage: ci/release/docker.sh [OPTIONS] vX.Y.Z COMMAND'
  echo
  echo 'Valid COMMANDs are:'
  echo ' * check: check the docker image can be built'
  echo ' * release: build the images from scratch (no caches) and pushes them'
  echo
  echo 'Valid OPTIONS are:'
  echo ' * --verbose: enable verbose logging'
}

while [[ $# -ne 0 ]]; do
  arg=$1
  shift

  case "${arg}" in
    v[0-9].[0-9].[0-9]) VERSON=$arg;;
    check|release)
      if [[ ! -z "${COMMAND}" ]]; then
        echo 'Specify only one of check or release'
        usage;
        exit 1
      fi
      COMMAND=$arg
      ;;
    --verbose) VERBOSE="yes";;

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
  echo 'Need a VERSON to build'
  usage
  exit 1
fi
if [[ -z ${COMMAND} ]]; then
  echo 'Need a COMMAND to run'
  usage
  exit 1
fi

# Check git status in release mode.
if [[ ${COMMAND} == "release" ]]; then
  if [[ ! -z "$(git status --porcelain)" ]]; then
    git status
    echo '==> Uncommitted changes to the working directory!'
    echo "==> Can't release docker images with an unclean tree"
    exit 1
  fi
fi

# Build and tag image.
echo '==> Building docker image'
full_tag="${DOCKE_REPO}:${VERSON}"
minor_tag="${DOCKE_REPO}:${VERSON%\.[0-9]}"
major_tag="${DOCKE_REPO}:${VERSON%\.[0-9]\.[0-9]}"
latest_tag="${DOCKE_REPO}:latest"
if [[ ${VERBOSE} == "yes" ]]; then
  echo '---> Image will be tagged with:'
  echo "--->   - ${full_tag}"
  echo "--->   - ${minor_tag}"
  echo "--->   - ${major_tag}"
  echo "--->   - ${latest_tag}"
fi

docker_cache=""
if [[ ${COMMAND} == "release" ]]; then
  docker_cache="--no-cache"
  if [[ ${VERBOSE} == "yes" ]]; then
    echo '---> Skipping docker cache for release build'
  fi
fi
docker build ${docker_cache} --force-rm \
  --tag "${full_tag}" \
  --tag "${minor_tag}" \
  --tag "${major_tag}" \
  --tag "${latest_tag}" \
  .

# Push images in release mode
if [[ ${COMMAND} == "release" ]]; then
  echo '==> Pushing docker images'
  docker push "${full_tag}"
  docker push "${minor_tag}"
  docker push "${major_tag}"
  docker push "${latest_tag}"
fi
