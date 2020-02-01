# Notes on Podman
The idea is to move from docker/docker-compose to Podman for a few reasons:

  * Rootless development environment.
  * Declarative YAML pods (podman play kube).
  * Makes the possibility of provisioning easier?

There are some downsides as well:

  * Networking only works with root containers.
  * podman play kube does not support envs or variables.
  * All pods will share the same network.
  * Pods must all have different names.


## The stub pod
Start it and delete it with:

```bash
podman play kube <FILE>
podman pod rm -f <POD_NAME>
```


## podman play kube for provisioning
Primarily around YAML templates, root containers, and label filters.
Seems to be very limited compared to what podman can do (labels, networks, varaibles, ...).


## Bash scripting of podman commands
Essentially manually implementing YAML translator.
Would only work on Linux (which is the case for Podman anyway).

Could control network (root requirement) and lables for discovery.
A script could create a full cluster "node" making adding/removing nodes
by manipuating pods similar to how VMs would work.

It is theoretically possible to pick unused/random ports for each pod and
expose them as the host so that root would not be required.

It may be easier/faster to write the script in python/other anyway if
advanced logic (cli parsing, port checking, string templating) is desired/needed.
