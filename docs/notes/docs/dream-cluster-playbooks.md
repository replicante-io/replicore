---
id: cluster-playbooks
title: Cluster coordinated playbooks
---

Extend the action system to introduce cluster-coordinated action sequences.
The main focus to start with is a safe, no downtime, rolling restart:

  1. Implement a "drain node" action and a "drained" state.
  2. Implement rolling restart playbook:
     1. Pick a node
     2. Drain it and wait for it to be drained
     3. Restart the node
     4. Un-drain the node
     5. Move on to next node


## Open questions

  * Node selection?
    * Pick the node with the least primary shards at every stage?
    * Random?
    * User decided order?
    * Allow user to choose method?
  * What to do with nodes that join the cluster?
  * What to do with nodes that leave the cluster?
  * What to do with nodes that leave the cluster while being processed?
  * What to do in case of error?
  * How to cancel actions?
  * Allow to choose parallelism levels?
    * Absolute nodes number
    * Percent of nodes (rounded down to a min of 1 node)
    * Always ensure quorum is maintained
  * How to deal with clusters that can't be processed with no downtime (1 node clusters, etc ...)?
