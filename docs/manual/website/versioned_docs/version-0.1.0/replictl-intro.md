---
id: version-0.1.0-replictl-intro
title: Introduction
sidebar_label: Introduction
original_id: replictl-intro
---

Replicante comes with a command line tool to help with maintenance and one-off tasks.

<blockquote class="info">

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
replictl 0.1.0 [b73969f8b19a17cd881f4e04741cdc764757b7ef; index and working directory tainted]
Replicante command line tool

USAGE:
    replictl [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help           Prints help information
        --no-progress    Do not show progress bars
    -V, --version        Prints version information

OPTIONS:
    -c, --config <FILE>        Specifies the configuration file to use [default: replicante.yaml]
        --log-level <LEVEL>    Specifies the logging verbosity [possible values: Critical, Error, Warning, Info, Debug]

SUBCOMMANDS:
    check       Perform checks on the system to find issues
    help        Prints this message or the help of the given subcommand(s)
    versions    Reports version information for various systems
```

## Subcommands

  * [`replictl check`](replictl-check.md)
  * [`replictl versions`](replictl-versions.md)
