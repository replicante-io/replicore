## Zookeeper
* Administration:
  * A cluster-unique name for the node: `serverId` value of the [`conf`](https://zookeeper.apache.org/doc/current/zookeeperAdmin.html#sc_zkCommands) command output.
  * Cluster name shared by all nodes: user defined in agent configuration.
  * Version information: from the output of either [`envi`](https://zookeeper.apache.org/doc/current/zookeeperAdmin.html#sc_zkCommands) or [`srvr`](https://zookeeper.apache.org/doc/current/zookeeperAdmin.html#sc_zkCommands) command.

* Clustering: zookeeper processes forming an ensable.

* Replication:
  * For each node, which shards are on the node: a single shard named `ensamble`.
  * For each shard on each node, what the role of the node is: `Mode` value of the [`srvr`](https://zookeeper.apache.org/doc/current/zookeeperAdmin.html#sc_zkCommands) command output.
  * For each non-primary shard on each node, the replication lag for the node: ???

* Sharding:
  * What is a shard: the entire ensamble.
  * For each shard, the time of the last operation: unavailable.

* Performance:
  * Number of clients connected: `Connections` attributes of the [`stat`](https://zookeeper.apache.org/doc/current/zookeeperAdmin.html#sc_zkCommands) command.
  * Number of read/writes: unavailabe (packets received/sent is available).
