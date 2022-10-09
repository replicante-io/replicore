<!-- markdownlint-disable MD022 MD024 MD032 -->
# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## Unreleased
### Changed
- Update dependencies.

## 0.2.0 - 2022-09-13
### Added
- Add `replidev deps init` as alias for `replidev deps initialise`.
- Enable pod to host networking by default.
- Play server supports configurable address for the playground agents.
- Pods and containers can resolve `PODMAN_HOSTNAME` to the `host.containers.internal` IP.
- Wrapper command for `curl`.

### Changed
- **BREAKING**: Certificates location moved around with EasyRSA rework.
- Replace custom `podman-host` alias with `slirp4netns`'s built in `host.containers.internal`.
- Replace deprecated `easypki` with [Easy-RSA](https://easy-rsa.readthedocs.io/en/latest/).

### Removed
- **BREAKING**: Don't set `PODMAN_IP` environment variable for `replidev deps`.
- **BREAKING**: Don't set `podman-host` as an IP alias.

## 0.1.0 - 2020-05-28
### Added
- Configurable podman command to use.
- HTTPS certificate (re)generation command.
- Initialise and clean up replicante core dependencies.
- Inject named ports allocated port into templating variables.
- List available `replidev deps` pods.
- List running playground nodes.
- Optional per-machine configuration override.
- Release check and automation started.
- Start and stop replicante core dependencies pods.
- Start playground Replicante Core stack.
- Start playground nodes as pods.
- Support custom variables and JSON objects in pod definitions.
