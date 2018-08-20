## Zookeeper
* Administration:
  * A cluster name shared by all nodes: user defined in agent configuration.
  * A cluster-unique name for the node: `serverId` value of the [`conf`](https://zookeeper.apache.org/doc/current/zookeeperAdmin.html#sc_zkCommands) command output.
  * Version information: from the output of either [`envi`](https://zookeeper.apache.org/doc/current/zookeeperAdmin.html#sc_zkCommands) or [`srvr`](https://zookeeper.apache.org/doc/current/zookeeperAdmin.html#sc_zkCommands) command.

* Clustering: zookeeper processes forming an ensable.

* Sharding: (A shard is the entire ensamble)
  * A shard ID: the cluster name.
  * [Optional] An indicator of when the last write operation happened (commit offset):
    * The replication lag unit (i.e, seconds, commits, ...): offset/zkid.
    * A commit offset value (as a 64-bits integer): the `Zkid` value of the [`srvr`](https://zookeeper.apache.org/doc/current/zookeeperAdmin.html#sc_zkCommands) command.

* Replication:
  * Which shards are on the node: a single shard named as the cluster.
  * For each shard, what the role on the node is: `Mode` value of the [`srvr`](https://zookeeper.apache.org/doc/current/zookeeperAdmin.html#sc_zkCommands) command output.
  * [Optional] For each non-primary shard, the replication lag: unavailable (need access to primary as well as local node).
