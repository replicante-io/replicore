## Kafka
* Administration:
  * A cluster-unique name for the node: extracted from the name of the [`kafka.server:type=app-info,id=ID`](https://kafka.apache.org/documentation/#monitoring) JMX MBean.
  * Cluster name shared by all nodes: user defined in agent configuration.
  * Version information: extracted from the `version` attribute of the [`kafka.server:type=app-info`](https://kafka.apache.org/documentation/#monitoring) JMX MBean.

* Clustering:
  * Kafka processes forming a set of brokers.
  * The required Zookeeper cluster is not considered part of the cluster but a cluster on its own.

* Replication:
  * For each node, which shards are on the node: need to consult each topic's partition map zookeeper node (`/brokers/topics/PARTITION`).
  * For each shard on each node, what the role of the node is: need to consult each topic's partition map zookeeper node (`/brokers/topics/PARTITION`).
  * For each non-primary shard on each node, the replication lag for the node: value of the [`kafka.server:type=FetcherLagMetrics,name=ConsumerLag,clientId=ReplicaFetcherThread-0-LEADER_ID,topic=TOPIC,partition=PARTITON_ID`](https://kafka.apache.org/documentation/#monitoring) JMX MBean, expressed as number of messages.

* Sharding:
  * What is a shard: a topic partition.
  * What is a shard ID: `TOPIC/PARTITION`.
  * For each shard, the time of the last operation: unavailable.
