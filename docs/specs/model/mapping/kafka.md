## Kafka
* Administration:
  * A cluster name shared by all nodes: user defined in agent configuration.
  * A cluster-unique name for the node: extracted from the name of the [`kafka.server:type=app-info,id=ID`](https://kafka.apache.org/documentation/#monitoring) JMX MBean.
  * Version information: extracted from the `version` attribute of the [`kafka.server:type=app-info`](https://kafka.apache.org/documentation/#monitoring) JMX MBean.

* Clustering:
  * Kafka processes forming a set of brokers.
  * The required Zookeeper cluster is not considered part of the cluster but a cluster on its own.

* Replication:
  * Which shards are on the node: need to consult each topic's partition map zookeeper node (`/brokers/topics/PARTITION`).
  * For each shard, what the role on the node is: need to consult each topic's partition map zookeeper node (`/brokers/topics/PARTITION`).
  * [Optional] For each non-primary shard, the replication lag:
    * The replication lag unit (i.e, seconds, commits, ...): number of messages.
    * The replication lag value (as a 64-bits integer): value of the [`kafka.server:type=FetcherLagMetrics,name=ConsumerLag,clientId=ReplicaFetcherThread-0-LEADER_ID,topic=TOPIC,partition=PARTITON_ID`](https://kafka.apache.org/documentation/#monitoring) JMX MBean.

* Sharding: (A shard is a topic partition).
  * A shard ID: `TOPIC/PARTITION`.
  * [Optional] An indicator of when the last write operation happened (commit offset):
    * A commit offset unit (i.e, seconds, commits, ...): offset.
    * A commit offset value (as a 64-bits integer): [topic offsets](https://docs.rs/kafka/0.7.0/kafka/client/struct.KafkaClient.html#method.fetch_offsets)
