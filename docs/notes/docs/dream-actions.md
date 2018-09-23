---
id: actions
title: The Actions System
---

This is in dreamland but is one of the two pillars or Replicante (monitoring is the other).

There is no value in designing the actions system until the monitoring layer is usable
and the high availability and scalability of the system are assessed.

Once that is done, some things must be included in the actions system:

  * Some form of guards/assertions: ensure node state has not changed since the time the action
    was scheduled for execution and the time it is actually executed.
  * Per-cluster coordination: to perform complex actions across multiple nodes without risk.
  * Cluster level actions: some actions may not apply to nodes but instead clusters or shards.
  * Actions must be signed/authenticated: must avoid unauthorised system from issuing commands!
  * Agents can disable actions: either all or some, on their side.
