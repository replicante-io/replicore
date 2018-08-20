## MongoDB Replica Set
* Administration:
  * A cluster-unique name for the node: name field from [`replSetGetStatus`](https://docs.mongodb.com/manual/reference/command/replSetGetStatus/).
  * Cluster name shared by all nodes: name field from [`replSetGetStatus`](https://docs.mongodb.com/manual/reference/command/replSetGetStatus/).
  * Version information: [`buildInfo`](https://docs.mongodb.com/manual/reference/command/buildInfo/).

* Clustering: `mongod` instances talking to each other.

* Replication:
  * For each node, which shards are on the node: a single shard named after the replica set.
  * For each shard on each node, what the role of the node is: [`replSetGetStatus`](https://docs.mongodb.com/manual/reference/command/replSetGetStatus/).
  * For each non-primary shard on each node, the replication lag for the node: [`replSetGetStatus`](https://docs.mongodb.com/manual/reference/command/replSetGetStatus/).

* Sharding:
  * What is a shard: the entire replica set.
  * What is a shard ID: RS name.
  * For each shard, the time of the last operation: [`replSetGetStatus`](https://docs.mongodb.com/manual/reference/command/replSetGetStatus/).

* Performance:
  * Number of clients connected: [`serverStatus`](https://docs.mongodb.com/manual/reference/command/serverStatus/).
  * Number of read/writes: [`serverStatus`](https://docs.mongodb.com/manual/reference/command/serverStatus/).


## MongoDB Sharded
* Administration:
  * A cluster-unique name for the node:
    * `mongod`: name field from [`replSetGetStatus`](https://docs.mongodb.com/manual/reference/command/replSetGetStatus/).
    * `mongos`: user defined in agent configuration.
  * Cluster name shared by all nodes: user defined in agent configuration.
  * Version information: [`buildInfo`](https://docs.mongodb.com/manual/reference/command/buildInfo/).

* Clustering:
  * `mongod` instances forming the configuration Replica Set.
  * `mongod` instances forming shard Replica Sets.
  * `mongos` instances routing queries.

* Replication:
  * For each node, which shards are on the node: a single shard named as the replica set.
  * For each shard on each node, what the role of the node is: [`replSetGetStatus`](https://docs.mongodb.com/manual/reference/command/replSetGetStatus/)
  * For each non-primary shard on each node, the replication lag for the node: [`replSetGetStatus`](https://docs.mongodb.com/manual/reference/command/replSetGetStatus/).

* Sharding:
  * What is a shard:
    * `mongod`: a shard is one of the Replica Sets storing the data.
    * `mongos`: `mongos` instances have no shards on them.
  * What is a shard ID: the shard's RS name.
  * For each shard, the time of the last operation: [`replSetGetStatus`](https://docs.mongodb.com/manual/reference/command/replSetGetStatus/).
