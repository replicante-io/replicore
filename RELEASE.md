# Replicante Core Release Steps

- Manual steps 1:
  - [ ] Bump the version number of all crates that need it
  - [ ] Update changelog with version and date
  - [ ] Update docs changelog
  - [ ] Update docs example configuration
- [ ] Scripted steps 1: `ci/release/prep.sh vX.Y.Z`
  - Ensure dependences are up to date
  - Ensure tests and CI checks pass
  - Update cargo lock file
  - Version documents
  - Ensure docker image builds correctly
- Manual steps 2:
  - [ ] Git commit and tag release
- [ ] Scripted steps 2: `ci/release/artefacts.sh vX.Y.Z`
  - Build and push docker image
  - Build pre-built binaries
- Manual steps 3:
  - [ ] Release pre-built binaries
  - [ ] Release documentation


## Pre-built binaries
Pre-built binaries can help speed up installations and reduce barrier to entry.
They **DO NOT** aim to be fully portable across major linux distributions.
In particular they use a relatively recent version of `glibc` that is unlikely
to be available on older or "slower" distributions (ubuntu 14.04, CentOS, ...).

The `ci/release/artefacts.sh` script will produce the pre-built binaries
to upload to GitHub in the `target/gh-releases` directory.
