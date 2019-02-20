---
id: version-0.2.0-replictl-coordinator
title: replictl coordinator
sidebar_label: replictl coordinator
original_id: replictl-coordinator
---

Interact with the distributed coordinator:

```text
$ replictl coordinator help
Inspect and manage cluster coordination

USAGE:
    replictl coordinator [FLAGS] [OPTIONS] [SUBCOMMAND]

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
    election    Inspect and manage distributed elections
    help        Prints this message or the help of the given subcommand(s)
    nb-lock     Inspect and manage distributed non-blocking locks
```


## election info
Show information about a specific election.

```bash
replictl coordinator election info ELECTION
```


## election ls
Show a list of elections currently running in the system.

```bash
replictl coordinator election ls
```


## election step-down
Step down the current primary for an election and forces a new election.

```bash
replictl coordinator election step-down ELECTION
```


## nb-lock force-release
Force the removal of a lock to other processes can acquire it.

```bash
replictl coordinator nb-lock force-release LOCK
```

<blockquote class="danger">

**This operation is dangerous!**

There is no reason for a lock to be force-released.
The coordinator can automatically release locks for processes that failed.

</blockquote>


## nb-lock info
Show information about a specific lock.

```bash
replictl coordinator nb-lock info LOCK
```


## nb-lock ls
Show a list of currently held locks in the system.

```bash
replictl coordinator nb-lock ls
```
