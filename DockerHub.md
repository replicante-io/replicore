# Overview
[Replicante](https://www.replicante.io/) is a centralised monitoring and management tool.

Replicante Core is the HA platform that monitors and orchestrates datastores.


## Usage
Replicante Core processes can be started with the command below.

Since a configuration file is required, one should be mounted or added to the image
by building a derived image.

```bash
docker run --rm -it \
  -v "/path/to/config.yaml:/home/replicante/replicante.yaml" \
  -w /home/replicante replicanteio/replicante:v0
```

See the tags for possible versions.
In addition to specific version, tags in the format `vX.Y` and `vX` refer to the latest
release for a specific minor version or a specific major version.
The tag `latest` is also available.


## More
For more information, the following links may be useful:

  * [Official website](https://www.replicante.io/)
  * [GitHub repo](https://github.com/replicante-io/replicante)
  * [Full documentation](https://www.replicante.io/docs/manual/docs/intro/)
