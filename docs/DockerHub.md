# Overview
[Replicante](https://www.replicante.io/) is a centralised datastore orchestrator.

Replicante Core is the HA platform that monitors and orchestrates datastores.


## Usage
Replicante Core processes can be started with the command below.

Since a configuration file is required, one should be mounted or added to the image
by building a derived image.

```bash
docker run --rm -it --init \
  -v "/path/to/config.yaml:/home/replicante/replicante.yaml" \
  replicanteio/replicante:v0
```

See the tags for possible versions.
In addition to the exact `vX.Y.Z` version, tags in the format `vX.Y` and `vX` refer to the
latest release for a specific minor version or a specific major version.
The tag `latest` is also available.


## Init on Podman
Podman defaults to [catatonit](https://github.com/openSUSE/catatonit) as the `--init` process.
This package is currently [not packaged](https://github.com/containers/libpod/issues/4159), at least for fedora.

Until catatonit is packaged with podman you can use this work around:

  1. Install a catatonit [release](https://github.com/openSUSE/catatonit/releases) or any container init process.
  2. Place it in a place podman will find it:
     * Podman looks at `/usr/libexec/podman/catatonit`
     * Set the `init_path` configuration option to your location.


## More
For more information, the following links may be useful:

  * [Official website](https://www.replicante.io/)
  * [GitHub repo](https://github.com/replicante-io/replicore)
  * [Full documentation](https://www.replicante.io/docs/)
