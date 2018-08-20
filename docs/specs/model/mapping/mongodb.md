## MongoDB Replica Set
* Administration:
  * A cluster name shared by all nodes: name field from [`replSetGetStatus`](https://docs.mongodb.com/manual/reference/command/replSetGetStatus/).
  * A cluster-unique name for the node: name field from [`replSetGetStatus`](https://docs.mongodb.com/manual/reference/command/replSetGetStatus/).
  * Version information: [`buildInfo`](https://docs.mongodb.com/manual/reference/command/buildInfo/).

* Clustering: `mongod` instances talking to each other.

* Sharding: (A shard is the entire replica set)
  * A shard ID: RS name.
  * [Optional] An indicator of when the last write operation happened (commit offset):
    * A commit offset unit (i.e, seconds, commits, ...): seconds (since epoch).
    * A commit offset value (as a 64-bits integer): [`replSetGetStatus`](https://docs.mongodb.com/manual/reference/command/replSetGetStatus/).

* Replication:
  * Which shards are on the node: a single shard named after the replica set.
  * For each shard, what the role on the node is: [`replSetGetStatus`](https://docs.mongodb.com/manual/reference/command/replSetGetStatus/).
  * [Optional] For each non-primary shard, the replication lag:
    * The replication lag unit (i.e, seconds, commits, ...): seconds.
    * The replication lag value (as a 64-bits integer): [`replSetGetStatus`](https://docs.mongodb.com/manual/reference/command/replSetGetStatus/).


## MongoDB Sharded
* Administration:
  * A cluster name shared by all nodes: user defined in agent configuration.
  * A cluster-unique name for the node:
    * `mongod`: name field from [`replSetGetStatus`](https://docs.mongodb.com/manual/reference/command/replSetGetStatus/).
    * `mongos`: user defined in agent configuration.
  * Version information: [`buildInfo`](https://docs.mongodb.com/manual/reference/command/buildInfo/).

* Clustering:
  * `mongod` instances forming the configuration Replica Set.
  * `mongod` instances forming shard Replica Sets.
  * `mongos` instances routing queries.

* Sharding:
  * A shard is ...:
    * `mongod`: a shard is one of the Replica Sets storing the data.
    * `mongos`: `mongos` instances have no shards on them.
  * A shard ID: the shard's RS name.
  * [Optional] An indicator of when the last write operation happened (commit offset):
    * A commit offset unit (i.e, seconds, commits, ...): seconds (since epoch).
    * A commit offset value (as a 64-bits integer): [`replSetGetStatus`](https://docs.mongodb.com/manual/reference/command/replSetGetStatus/).

* Replication:
  * Which shards are on the node: a single shard named as the replica set.
  * For each shard, what the role on the node is: [`replSetGetStatus`](https://docs.mongodb.com/manual/reference/command/replSetGetStatus/)
  * [Optional] For each non-primary shard, the replication lag:
    * The replication lag unit (i.e, seconds, commits, ...): seconds.
    * The replication lag value (as a 64-bits integer): [`replSetGetStatus`](https://docs.mongodb.com/manual/reference/command/replSetGetStatus/).
