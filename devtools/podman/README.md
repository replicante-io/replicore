# Notes on Podman
The idea is to move from docker/docker-compose to Podman for a few reasons:

  * Rootless development environment.
  * Pods support (which is not the same as docker-compose).

There are some downsides as well:

  * Networking only easily works with root containers.
  * podman play kube lucks needed features (for now) so needed a custorm wrapper.
  * Pods must all have different names (a bit of a pain in playgrounds).


## podman play kube for provisioning
Describe pods as YAML files like in Kubernetes but not all features are available.
Seems to be very limited compared to what podman CLI can do:

  * Pod annotations/labels are not available.
  * Container annotations/labels are not available.
  * Networks configuration (rootless makes this a non-point though)
  * Environment varaibles or other customisation (needed for use as dev tool).


## Bash scripting of podman commands
Essentially implementing YAML translator in bash.
Would likely focus on Linux (which is the case for podman anyway).

Could control network (root requirement) and lables for discovery.
A script could create a full cluster "node" making adding/removing nodes
by manipuating pods similar to how VMs could work.

It is theoretically possible to pick unused/random ports for each pod and
expose them as the host so that root would not be required.

It may be easier/faster to write the script in python/other anyway if
advanced logic (cli parsing, port checking, string templating) is desired/needed.


## Replidev deps
This is the solution I landed on in the end: a custom made tool that
translate a YAML file into podman commands end executes them.

Has the advantages of `podman play kube` but with needed features on top
(variables, annotations/labels).

It comes at the cost of managing a custom built tool with large overlaps
with other existing tools.
On the other hand the tool was likely to be introduced regardless as
many scripts, wrappers and other things are added to automate bits.
