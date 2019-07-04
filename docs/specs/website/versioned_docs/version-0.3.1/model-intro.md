---
id: version-0.3.1-model
title: The Datastore Model
sidebar_label: The Datastore Model
original_id: model
---

<blockquote class="warning">

**Alpha state disclaimer**

The model defined below is in early development cycle
and is subject to (potentially breaking) change.

</blockquote>

Any (collection of) software that fulfils the requirements and expectations
of the model defined in this document is considered a datastore.

The difficulty in defining a model is to find the balance
between generality and specificity:

  * The model should be general so it does not impose restrictions on the
    datastores that want to be supported (or on us to support datastores).
  * The model also should be specific enough so that it can be reliably operated on.

This aims of the model are:

  * To determine which software can be modelled and how.
  * To implement agents that sit between the datastore and the central system.
  * To detail what information is available and what actions can be performed by the agents.
  * To determine what features can be built on top of this standardised layer.


## Administration
The datastore MUST provide the following administration information:

  * A cluster ID shared by all nodes.
  * A cluster-unique name for the node.
  * Version information.

The datastore MAY provide the following administration information:

  * An operator friendly display name for the cluster.


## Clustering
The datastore MUST support clustering by running a process on one or more (virtual or physical)
machine.
Each process in the cluster is a node.

Note that there is no requirement for the process be the same everywere in the cluster
(same applies to how nodes are configured).
This allows the cluster to have heterogeneous components as long as they all follow the model.


## Sharding
The datastore MUST organize the data in one or more shard.
Shards are independent units of data, each with their own primary and secondaries nodes.

All datastores have at least one shard.  
For datastores that do not support sharding, the entire dataset can be seen as a single shard.

For each shard in the cluster the datastore MUST provide the following information:

  * A shard ID.

Each node in the cluster SHOULD provide the following information for each:

  * An indicator of when the last write operation happened (commit offset):
    * A commit offset unit (i.e, seconds, commits, ...).
    * A commit offset value (as a 64-bits integer).


## Replication
The datastore MUST support a primary/secondaries replication system.
This means that each shard at any given time has only primary node with zero or more
secondary nodes that replicate the data from another node.

Each node in the cluster MUST provide the following information:

  * Which shards are on the node.
  * For each shard, what the role on the node is (i.e, primary, secondary).

Some details about replication require the cluster to be healthy enough to report such data.
Such details may also be expensive to compute or, worse, require connections to non-local nodes.

This information should be provided whenever possible as long as:

  * Computing the information only requires local information
    (i.e, it does not require connections to other nodes).
  * The information can be computed relatively efficiently.

Each node in the cluster SHOULD provide the following information:

  * For each non-primary shard, a replication lag:
    * The replication lag unit (i.e, seconds, commits, etc ...).
    * The replication lag value.
    * It is NOT required for the unit to be the same as the commit offset but it is encouraged.
