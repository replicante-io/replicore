#!/bin/env bash
set -e

# Constants.
DOCKER_ORG="replicanteio"

# Confguration variables.
cmd=""
docker="podman"
docker_repo="replicante"
sudo=""
version=""
verbose="no"

# Parse CLI args.
usage() {
  echo 'Usage: ci/release/docker.sh [OPTIONS] VERSION COMMAND'
  echo
  echo 'VERSION is the Replicante Core version to build and is in the format vX.Y.Z'
  echo
  echo 'Available commands:'
  echo '  check      Check the docker image can be built'
  echo '  release    Build the images from scratch (no caches) and pushes them'
  echo
  echo 'Available options:'
  echo '  --repo REPO    The docker repository to build images for'
  echo '  --sudo         Use sudo for docker commands'
  echo '  --verbose      Enable verbose logging'
}

while [[ $# -ne 0 ]]; do
  arg=$1
  shift

  case "${arg}" in
    v[0-9].[0-9].[0-9]) version=$arg;;
    check|release)
      if [[ ! -z "${cmd}" ]]; then
        echo 'Specify only one of check or release'
        usage;
        exit 1
      fi
      cmd=$arg
      ;;
    --repo) docker_repo=$1; shift 1;;
    --sudo) sudo="sudo";;
    --verbose) verbose="yes";;

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
  echo 'Need a version to build'
  usage
  exit 1
fi
if [[ -z ${cmd} ]]; then
  echo 'Need a command to run'
  usage
  exit 1
fi

# Check git status in release mode.
if [[ ${cmd} == "release" ]]; then
  if [[ ! -z "$(git status --porcelain)" ]]; then
    git status
    echo '==> Uncommitted changes to the working directory!'
    echo "==> Can't release docker images with an unclean tree"
    exit 1
  fi
fi

# Build and tag image.
echo '==> Building docker image'
full_tag="${DOCKER_ORG}/${docker_repo}:${version}"
minor_tag="${DOCKER_ORG}/${docker_repo}:${version%\.[0-9]}"
major_tag="${DOCKER_ORG}/${docker_repo}:${version%\.[0-9]\.[0-9]}"
latest_tag="${DOCKER_ORG}/${docker_repo}:latest"
if [[ ${verbose} == "yes" ]]; then
  echo '---> Image will be tagged with:'
  echo "--->   - ${full_tag}"
  echo "--->   - ${minor_tag}"
  echo "--->   - ${major_tag}"
  echo "--->   - ${latest_tag}"
fi

docker_cache=""
if [[ ${cmd} == "release" ]]; then
  docker_cache="--no-cache"
  if [[ ${verbose} == "yes" ]]; then
    echo '---> Skipping docker cache for release build'
  fi
fi
${sudo} ${docker} build ${docker_cache} --force-rm --format docker \
  --tag "${full_tag}" \
  --tag "${minor_tag}" \
  --tag "${major_tag}" \
  --tag "${latest_tag}" \
  .

# Push images in release mode
if [[ ${cmd} == "release" ]]; then
  echo '==> Pushing docker images'
  ${sudo} ${docker} push "${full_tag}"
  ${sudo} ${docker} push "${minor_tag}"
  ${sudo} ${docker} push "${major_tag}"
  ${sudo} ${docker} push "${latest_tag}"
fi
