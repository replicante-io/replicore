# Replicante Development helper
A tool to speed up development of Replicante Core.

By having strong opinions about various development tasks, `replidev` can leverage existing
general purpose tools to automate the repetitive tasks that occur during development of
Replicante Core, its agents and any other common task.

Its main goal is to give you the tools you need to focus on development.

## Certificates
Various projects need client or server TLS certificates for HTTPS.

These can be generated with `replidev gen-certs` and will be valid for a year.
You can run with `--regen` to delete and re-create all certificates for the repo you are in.

> Requires [EasyRSA] and [OpenSSL] in `$PATH`.

## Core Development Environment
The `replidev deps` family of commands is there to manage Replicante Core dependencies
as well as optional development tools that may be useful at times.

All commands operate on one or more dependency `POD`.
The list of running pods and known pods that can be started is available from the
`replidev deps list` command.

> Requires [Podman] version 1.9 or later in `$PATH`.

## Playgrounds
All things related to local demo/test/development clusters, agents and core.

Check out the [quick start](https://www.replicante.io/quick-start/) for all the details.

> Requires [Podman] version 1.9 or later in `$PATH`.

[EasyRSA]: <https://github.com/OpenVPN/easy-rsa>
[OpenSSL]: <https://www.openssl.org/>
[Podman]: <https://podman.io/>
