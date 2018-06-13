# System checks
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
    -c, --config <FILE>        Specifies the configuration file to use [default: replicante.yaml]
        --log-level <LEVEL>    Specifies the logging verbosity [possible values: Critical, Error, Warning, Info, Debug]

SUBCOMMANDS:
    config    Check the replicante configuration for errors
    deep      Run all checks INCLUDING the ones that iterate over ALL data
    help      Prints this message or the help of the given subcommand(s)
    quick     Run all checks that do NOT iterate over data (default command)
    store     Check the primary store for incompatibilities
    update    Run all checks to confirm an update is possible (iterates over ALL data!)
```


## config
Parse the given replicante configuration to check all attributes and referenced files can be loaded.

```bash
replictl check config
```


## deep
An alias to run all checks.

```bash
replictl check deep
```

{% hint style="danger" %}
**This check will scan the full content of the storage layer**

As a result it may take a while to complete and could impact performance on large instances.  
Be careful when using this command against an active replicante system.
{% endhint %}


## quick
An alias to run all checks except for those that access the storage layer.

```bash
replictl check quick
```


## store data
Scan every item in the store to look for incompatible data.
This can reveal issues linked to running a version of replicante incompatible with the data.

```bash
replictl check store data
```

{% hint style="danger" %}
**This check will scan the full content of the storage layer**

As a result it may take a while to complete and could impact performance on large instances.  
Be careful when using this command against an active replicante system.
{% endhint %}

## store schema
Checks the storage layer metadata for required and suggested items (collections, indexes, tables, ...).

Also checks for the existence of items (collections, indexes, tables, ...) that belong to older versions of replicante.

```bash
replictl check store schema
```


## update
An alias to run all checks to ensure the replicante version linked to this version of `replictl`
is compatible with the current setup (checks if an in-place update is possible).

```bash
replictl check update
```

{% hint style="danger" %}
**This check will scan the full content of the storage layer**

As a result it may take a while to complete and could impact performance on large instances.  
Be careful when using this command against an active replicante system.
{% endhint %}
