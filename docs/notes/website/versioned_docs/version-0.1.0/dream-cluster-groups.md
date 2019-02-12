---
id: version-0.1.0-cluster-groups
title: Cluster groups
original_id: cluster-groups
---

Make Replicante aware of cluster dependences?
An example would be kafka that relies on zookeeper.

The advantage is that issues detected on Kafka while zookeeper is down can be
correctly attributed to zookeeper and not kafka.
