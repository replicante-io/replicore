---
id: scaling
title: Scaling
---

Replicante is an highly available and scalable service.

## High availability
High availability is achieved by running multiple instances of the same process.
Replicante uses a coordinator service (Zookeeper) to ensure that processes do not
interfere with each other and that tasks assigned to failed process are taken over.


## Scaling
Replicante using sharding techniques to scale: each cluster is a "shard" of data.

Replicante assumes a cluster can be managed by a single process.
The code attempts to process each node individuality in a lazy fashion.


## Dependencies
All services Replicante depends on can be configured to be highly available and scale as well.
It is the user's task to configure all services to provide high availability and scale as desired.

Dependency services that support sharding should be configured to shard data on cluster ID.  
**NOTE: this is likely to change!!!**

  * Cluster IDs (unique; machine usable) vs Cluster Names (unique?; human readable; think label)
    are likely to be implemented at some point.
    This feature could change the name of fields in the store.
  * Organisations may be introduced sooner or later, allowing Replicante to become a multi-tenanted
    platform, which would likely change the shard key from cluster ID to organisation ID
    (plus cluster ID for instances that need to deal with large organisations).