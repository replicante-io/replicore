# The Datastore Model
> **[warning] Alpha state disclaimer**
>
> The model defined below is in early development cycle
> and is subject to (potentially breaking) change.

Any (collection of) software that fullfills the requirements and expectations
of the model defined in this document is considered a datastore.

The difficulty in defining a model is to find the balance
between generality and specificity:

  * The model should be general so it does not impose restirctions on the
    datastores that want to be supported (or on us to support datastores).
  * The model also should be specific enough so that it can be operated on.

This aims of the model are:

  * To detemine which softwares can be modelled and how.
  * To implement agents that sit between the datastore and the central system.
  * To detail what information is available and what actions can be performed
    by the agents.
  * To determine what features can be built on top of this standardised layer.


## Administration
The datastore MUST provide the following administration commands:

  * A cluster-unique name for the node.
  * Version information.

The datastore MAY provide the following administration commands:

  * Cluster name shared by all nodes.


## Clustering
The datastore MUST support clustering by running a
process on one or more (virtual or physical) machines.
Each process in the cluster is a node.

Note that there is no requirement for the process be the same everywere
in the cluster (same applies to nodes configuration).
This allows the cluster to have heterogeneous components as long as they
all follow the model.


## Replication
The datastore MUST support a primary/secondaries replication system.
This means that each shard (see below) at any given time has only primary node
with zero or more secondary nodes that replicate the data from another node.

The datastore MUST provide the following information:

  * For each node, which shards are on the node.
  * For each shard on each node, what the role of the node is
    (i.e, primary for the shard).

Some details about replication require the cluster to be healthy enough to report such data.
Such details may also be expensive to compute or, worse, require connections to non-local nodes.

This information should be provided whenever possible as long as:

  * Computing the information only requires local information
    (i.e, it does not require connections to other nodes).
  * The information can be computed relatively efficiently.

The datastore SHOULD provide the following information:

  * For each non-primary shard on each node, the replication lag for the node.
    The lag must be at least at the second granularity.


## Sharding
The datastore MUST organize the data in one or more shard.
Shards are independent units of data, each with their own primary and
secondaries nodes.

All datastores have at least one shard: the entire dataset.

The datastore MUST provide the following information:

  * For each shard, the time of the last operation.


## Performance
The datastore SHOULD provide some performance metrics for users to understand
the overall state of the system.

The datastore SHOULD provide the following metrics:

  * Number of clients connected.
  * Number of read/writes.
