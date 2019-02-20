---
id: version-0.2.0-replictl-intro
title: Introduction
sidebar_label: Introduction
original_id: replictl-intro
---

Replicante comes with a command line tool to help with maintenance and one-off tasks.

<blockquote class="warning">

The features described in this section strongly depend on the version
of `replictl` being the same as the version of Replicante core.

</blockquote>

<blockquote class="info">

Some of the features in `replictl` need access to the replicante core configuration and services.

For example, the [`replictl check schema`](replictl-check.md#schema) command will need access
to the configured storage layer to operate.

</blockquote>

`replictl` offers a sub-command based structure like [git](https://git-scm.com/)
and [cargo](https://doc.rust-lang.org/cargo/index.html):

```text
$ replictl help
replictl 0.2.0 [b6f6fabed66cc046746cbb7a1cfa59f50cf666c1; index and working directory tainted]
Replicante command line tool

USAGE:
    replictl [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help           Prints help information
        --no-progress    Do not show progress bars
    -V, --version        Prints version information

OPTIONS:
    -c, --config <FILE>             Specifies the configuration file to use [default: replicante.yaml]
        --log-level <LEVEL>         Specifies the logging verbosity [possible values: Critical, Error, Warning, Info,
                                    Debug]
        --progress-chunk <CHUNK>    Specifies how frequently to show progress messages [default: 500]
        --url <URL>                 Specifies the URL of the Replicante API to use [default: http://localhost:16016/]

SUBCOMMANDS:
    check          Perform checks on the system to find issues
    coordinator    Inspect and manage cluster coordination
    help           Prints this message or the help of the given subcommand(s)
    versions       Reports version information for various systems
```

## Subcommands

  * [`replictl check`](replictl-check.md)
  * [`replictl coordinator`](replictl-coordinator.md)
  * [`replictl versions`](replictl-versions.md)
