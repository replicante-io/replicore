---
id: version-0.2.0-scaling-store
title: Primary Store
sidebar_label: Primary Store
original_id: scaling-store
---

Scaling the primary store is generally left to the primary store itself.
Available options vary based on the selected backend.


## MongoDB
[MongoDB](https://www.mongodb.com/) is the currently recommended backend for the primary store.

A single replica set should be able to handle most installations.
For cases where it is not, MongoDB provides tansparent
[sharding](https://docs.mongodb.com/manual/sharding/) support with the use of `mongos`.

<blockquote class="warning">

Replicante Core uses a [pure-rust mongodb driver](https://crates.io/crates/mongodb)
that is currently in prototype stage.

While this has not been a problem yet, some issues with performance and/or advanced setups may arise.
If that is the case [please report](https://github.com/replicante-io/replicante/issues) them.

</blockquote>


MongoDB sharding works by dividing the data across different (shard) replica sets.
To know how data should be divided and what data is needed by each query, MongoDB needs a
[shard key](https://docs.mongodb.com/manual/sharding/#shard-keys).

The selection of a good shard key is critical to the performance of a sharded cluster
and depends on the exact schema of the data stored in each collection.

<blockquote class="danger">

Once a key is selected it is not possible (read: complex and expensive) to change the shard
key of a collection.

As Replicante Core is at the beginning of its path, several features are likely to result in
changes to the schema of the stored data, **including to the optimal sharding key**.

Installations that want to make use of sharded clusters should carefully consider what their
upgrade paths will be if/when the sharding key changes.

</blockquote>

As of version 0.2.0, the suggested sharding keys for each collection are below:

  * `agents`: `(cluster: 1, host: 1)`
  * `agents_info`: `(cluster: 1, host: 1)`
  * `clusters_meta`: `name: 1`
  * `discoveries`: `name: 1`
  * `nodes`: `(cluster: 1, name: 1)`
  * `shards`: `(cluster: 1, node: 1, id: 1)`


<blockquote class="info">

If you do run a Replicante cluster and plan to use sharding we would be very pleased to know.

Please let us know by [opening an issue](https://github.com/replicante-io/replicante/issues).

</blockquote>
