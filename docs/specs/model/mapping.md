# Examples: mapping DBs to the Model
The sections below shows examples of how some popular software map onto the datastore model.


## MongoDB Replica Set
* Administration:
  * A cluster-unique name for the node: name field from [`replSetGetStatus`](https://docs.mongodb.com/manual/reference/command/replSetGetStatus/).
  * Version information: [`buildInfo`](https://docs.mongodb.com/manual/reference/command/buildInfo/).

* Clustering: `mongod` instances talking to each other.

* Replication:
  * For each node, which shards are on the node: a single shard named after the replica set.
  * For each shard on each node, what the role of the node is: [`replSetGetStatus`](https://docs.mongodb.com/manual/reference/command/replSetGetStatus/).
  * For each non-primary shard on each node, the replication lag for the node: [`replSetGetStatus`](https://docs.mongodb.com/manual/reference/command/replSetGetStatus/).

* Sharding:
  * What is a shard: the entire replica set.
  * For each shard, the time of the last operation: [`replSetGetStatus`](https://docs.mongodb.com/manual/reference/command/replSetGetStatus/).

* Performance:
  * Number of clients connected: [`serverStatus`](https://docs.mongodb.com/manual/reference/command/serverStatus/).
  * Number of read/writes: [`serverStatus`](https://docs.mongodb.com/manual/reference/command/serverStatus/).


## MongoDB Sharded
* Administration:
  * A cluster-unique name for the node: name field from [`replSetGetStatus`](https://docs.mongodb.com/manual/reference/command/replSetGetStatus/).
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
  * What is a shard: a shard is one of the Replica Sets storing the data.
  * For each shard, the time of the last operation: [`replSetGetStatus`](https://docs.mongodb.com/manual/reference/command/replSetGetStatus/).

* Performance:
  * Number of clients connected: [`serverStatus`](https://docs.mongodb.com/manual/reference/command/serverStatus/).
  * Number of read/writes: [`serverStatus`](https://docs.mongodb.com/manual/reference/command/serverStatus/).


## RabbitMQ
TODO: create a docker cluster for tests and check what the API returns.
Refenrence: https://cdn.rawgit.com/rabbitmq/rabbitmq-management/v3.7.2/priv/www/api/index.html

* Administration:
  * A cluster-unique name for the node: ???
  * Version information: ???

* Clustering: ???

* Replication:
  * For each node, which shards are on the node: ???
  * For each shard on each node, what the role of the node is: ???
  * For each non-primary shard on each node, the replication lag for the node: ???

* Sharding:
  * What is a shard: ???
  * For each shard, the time of the last operation: ???

* Performance:
  * Number of clients connected: ???
  * Number of read/writes: ???
