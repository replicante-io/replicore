---
id: scaling
title: Scaling
---

Replicante is a highly available and scalable service.


## High availability
High availability is achieved by running multiple instances of the same process.
Replicante uses a coordinator service (Zookeeper) to ensure that processes do not
interfere with each other and that tasks assigned to failed process are taken over.


## Scaling
Replicante uses sharding techniques to scale: each cluster is a "shard" operated on independently.

Replicante assumes a cluster can be managed by a single process.
The code attempts to process each node individuality in a lazy fashion when possible
to further reduce the impact of processing large clusters.

Details of scaling are provided in the
[admin manual](https://www.replicante.io/docs/manual/docs/scaling/).


## Dependencies
All services Replicante depends on can be configured to be highly available and scale as well.
It is the user's task to configure all services to provide high availability and scale as desired.

Dependency services that support sharding should be configured to shard data on cluster ID.  
**NOTE: this is likely to change!!!**

  * Organisations may be introduced sooner or later, allowing Replicante to become a multi-tenanted
    platform, which would likely change the shard key from cluster ID to organisation ID
    (plus cluster ID for instances that need to deal with large organisations).
