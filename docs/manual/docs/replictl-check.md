---
id: replictl-check
title: replictl check
sidebar_label: replictl check
---

Perform various checks on different aspects of the system.

```text
$ replictl check help
replictl-check
Perform checks on the system to find issues

USAGE:
    replictl check [FLAGS] [OPTIONS] [SUBCOMMAND]

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
    config         Check the replicante configuration for errors
    coordinator    Check all coordination data for incompatibilities
    deep           Run all checks INCLUDING the ones that iterate over ALL data
    help           Prints this message or the help of the given subcommand(s)
    quick          Run all checks that do NOT iterate over data (default command)
    stores         Check stores for incompatibilities
    streams        Check all streams for incompatibilities
    tasks          Check commands for the tasks subsystem
    update         Run all checks to confirm an update is possible (iterates over ALL data!)
```


## config
Parse the given replicante configuration to check all attributes and referenced files can be loaded.

```bash
replictl check config
```


## coordinator
Scan every item in the coordinator to look for schema incompatibilities.

Useful to reveal schema conflicts between the version of the currently running Replicante cluster
and the version of Replicante compiled against the `replictl` version.

```bash
replictl check coordinator
```

<blockquote class="danger">

**This check will scan the full content of the coordinator layer**

As a result it may take a while to complete and could impact performance on large instances.  
Be careful when using this command against an active replicante system.

</blockquote>


## deep
An alias to run all checks.

```bash
replictl check deep
```

<blockquote class="danger">

**This check will scan the full content of the storage layer**

As a result it may take a while to complete and could impact performance on large instances.  
Be careful when using this command against an active replicante system.

</blockquote>


## quick
An alias to run all checks except for those that access the storage layer.

```bash
replictl check quick
```


## stores primary data
Scan every item in the store to look for incompatible data.

Useful to reveal schema conflicts between the version of the currently running Replicante cluster
and the version of Replicante compiled against the `replictl` version.

```bash
replictl check stores primary data
```

<blockquote class="danger">

**This check will scan the full content of the storage layer**

As a result it may take a while to complete and could impact performance on large instances.  
Be careful when using this command against an active replicante system.

</blockquote>


## stores primary schema
Checks the storage layer metadata for required and suggested items (collections, indexes, tables, ...).

Also checks for the existence of items (collections, indexes, tables, ...) that belong to older versions of replicante.

```bash
replictl check stores primary schema
```


## streams events
Scan every item in the `events` stream to look for schema incompatibilities.

Useful to reveal schema conflicts between the version of the currently running Replicante cluster
and the version of Replicante compiled against the `replictl` version.

```bash
replictl check streams events
```

<blockquote class="danger">

**This check will scan the full content of the `events` stream layer**

As a result it may take a while to complete and could impact performance on large instances.  
Be careful when using this command against an active replicante system.

</blockquote>


## tasks data
Scan every item in the task queues to look for schema incompatibilities.

Useful to reveal schema conflicts between the version of the currently running Replicante cluster
and the version of Replicante compiled against the `replictl` version.

```bash
replictl check tasks data
```

<blockquote class="danger">

**This check will scan the full content of the task queues layer**

As a result it may take a while to complete and could impact performance on large instances.  
Be careful when using this command against an active replicante system.

</blockquote>


## update
An alias to run all checks to ensure the replicante version linked to this version of `replictl`
is compatible with the current setup (checks if an in-place update is possible).

```bash
replictl check update
```

<blockquote class="danger">

**This check will scan the full content of the storage layer**

As a result it may take a while to complete and could impact performance on large instances.  
Be careful when using this command against an active replicante system.

</blockquote>
