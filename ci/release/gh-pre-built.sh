#!/usr/bin/env bash
set -e


usage() {
  echo 'Usage: ci/release/gh-pre-built.sh [OPTIONS] VERSION'
  echo
  echo 'VERSION is the version to collect binaries for and is in the format vX.Y.Z'
  echo
  echo 'Available options:'
  echo '  --clean         Delete the content of the target directory first'
  echo '  --repo REPO     The docker repository to pull images from'
  echo '  --sudo          Use sudo for docker commands'
  echo '  --target DIR    Collect binaries in DIR target directory'
}


# Constants.
CHECKSUM_FILE="checksum.txt"
DOCKER_CONTAINER_NAME="replicante-gh-releases"
DOCKER_ORG="replicanteio"

# Config variables.
clean="false"
docker="podman"
docker_repo="replicante"
sudo=""
target="target/gh-releases"
version=""


# Figure out what we are asked to do.
while [ $# -ne 0 ]; do
  arg=$1
  shift

  case "${arg}" in
    --clean) clean="true";;
    --repo) docker_repo=$1; shift 1;;
    --sudo) sudo="sudo";;
    --target) target=$1; shift 1;;
    v[0-9]\.[0-9]\.[0-9]) version=${arg};;

    --help|help)
      usage
      exit 0
      ;;

    *)
      usage
      exit 1
      ;;
  esac
done

if [ -z "${version}" ]; then
  usage
  exit 1
fi


# Prepare target directory.
if [ "${clean}" == "true" ]; then
  echo "--> Cleaning target directory '${target}' first"
  rm -rf "${target}"
fi
mkdir -p "${target}"


# Create a container with the binaries we need to extract.
echo "--> Preparing docker environment"
docker_image="${DOCKER_ORG}/${docker_repo}:${version}"
${sudo} ${docker} pull "${docker_image}"
${sudo} ${docker} run --rm -id --name "${DOCKER_CONTAINER_NAME}" "${docker_image}" cat
trap "echo '--> Cleaning up docker environment'; ${docker} rm -f ${DOCKER_CONTAINER_NAME}" EXIT INT TERM


# Fetch binaries from available images.
extract_file() {
  local from_file="${DOCKER_CONTAINER_NAME}:/opt/replicante/bin/$1"
  local to_file="${target}/$1-linux-64bits"
  echo "Extracting '$1': ${from_file} --> ${to_file}"
  ${sudo} ${docker} cp "${from_file}" "${to_file}"
  ${sudo} chown "$(id -un):$(id -gn)" "${to_file}"
}

echo "--> Collecting binaries for version ${version}"
case "${docker_repo}" in
  agent-kafka)
    echo '--> Bundling kafka agent dependencies'
    ${sudo} ${docker} exec -it "${DOCKER_CONTAINER_NAME}" tar \
      --create --gzip \
      --file /home/replicante/replicante-agent-kafka.tar.gz \
      -C /opt/replicante/bin .
    ${sudo} ${docker} exec -it --user root "${DOCKER_CONTAINER_NAME}" cp \
      /home/replicante/replicante-agent-kafka.tar.gz \
      /opt/replicante/bin/replicante-agent-kafka.tar.gz
    extract_file 'replicante-agent-kafka.tar.gz'
    ;;

  agents)
    extract_file 'replicante-agent-mongodb'
    extract_file 'replicante-agent-zookeeper'
    ;;

  replicante)
    extract_file 'replicante'
    extract_file 'replictl'
    ;;

  *)
    echo "*** Unsupported repository: ${docker_repo} ***"
    exit 1
    ;;
esac

# Checksum file to verify all binaries.
pushd "${target}"
rm -f "${CHECKSUM_FILE}"
sha256sum * | tee "${CHECKSUM_FILE}"
popd
