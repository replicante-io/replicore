---
id: version-0.3.0-features-discovery
title: Cluster Discovery
sidebar_label: Cluster Discovery
original_id: features-discovery
---

Cluster discovery helps automation and reduces management overhead
(it may even help standardise your setup).

Cluster discovery follows the principle of single source of truth: nodes are looked up in
provisioning systems or other forms of inventory solutions.

Administrators only have to manage one list of nodes and replicante can use that information.
Combine this with infrastructure as a service solutions and you automatically get datastore
monitoring and automation as soon as a new server/instance is created.


## Backends
Replicante can support a variety of ways to discover clusters from different systems.
These are discovery *backends*.

Each backend has a set of [configuration](admin-config.md) options detailed below,
all set under the `discovery.backends` section.


### File backend
The file backend periodically reads clusters out of YAML files.

These files can be statically generated for clusters that rarely change or they can be
programmatically generated by many automation tools, `cron` jobs or other solutions.

The list of files to (periodically) read is set in the `discovery.backends.files` configuration option.

The content of the file is a list of objects as follows:

  * `cluster_id`: The ID of the cluster being discovered.
  * `display_name` (optional): A human friendly name for the cluster.
  * `nodes`: A list of addresses pointing to agents, one for each node in the cluster.

```yaml
# Example of two MongoDB replica sets
- cluster_id: cluster01
  display_name: 'Primary Cluster'
  nodes:
    - 'http://node1.example.com:37017'
    - 'http://node2.example.com:37017'
    - 'http://node3.example.com:37017'

- cluster_id: cluster02
  nodes:
    - 'http://node4.example.com:37017'
    - 'http://node5.example.com:37017'
    - 'http://node6.example.com:37017'
```


## Interval
Replicante Core periodically scans configured backends to detect changes to the
set of agents that should be monitored.

The `discovery.interval` option sets the delay, in seconds, between each scan.


## Cluster IDs and Display Names
Within the system, clusters are identified by a **unique ID**.
Cluster IDs are automatically extracted from the datastore if it has such unique ID.

This happens for example:

  * With MongoDB replica sets, which have a configured ID.
  * With Kafka, were the cluster generates a random ID when initialising.

For cases like MongoDB where it is configured or where the datastore does not provide such
IDs and they must be set in the agent configuration these IDs have a meaning for operators.
For cases like Kafka where the ID is automatically generated by the system
these IDs may be problematic for operators to work with.

Replicante supports display names for cases where IDs alone are too confusing.
Display names are used in the [WebUI](features-webui.md) to identify and search clusters.

<blockquote class="info">

Like cluster IDs, dispaly names must be unique to a single cluster and all agents
must report the same cluster ID and display name for nodes in the same cluster.

</blockquote>