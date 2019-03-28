# Releasing Core

- [ ] Ensure dependences are up to date
- [ ] Ensure tests and CI checks pass
- [ ] Bump the version number of all crates that need it
- [ ] Update cargo lock file
- [ ] Update changelog with version and date
- [ ] Update docs changelog
- [ ] Version documents
- [ ] Ensure docker image builds correctly
- [ ] Git commit and tag release
- [ ] Build and push docker image
- [ ] Release pre-built binaries
- [ ] Release documentation


## Pre-built binaries
Pre-built binaries can help speed up installations and reduce barrier to entry.
They **DO NOT** aim to be fully portable across major linux distributions.
In particular they use a relatively recent version of `glibc` that is unlikely
to be available on older or "slower" distributions (ubuntu 14.04, CentOS, ...).

To collect the pre-built binaries, once docker images have been published:
```bash
ci/gh-releases.sh --clean vX.Y.Z
ci/gh-releases.sh --clean --repo agents vX.Y.Z
ci/gh-releases.sh --repo agent-kafka vX.Y.Z
```
