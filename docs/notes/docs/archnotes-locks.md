---
id: coordinator
title: Distributed Coordination
---

Replicante uses a distributed coordinator for a variety of reasons.

This page aims to keep track of all uses of coordination.
Distributed coordination (especially locks) is a delicate thing, and are *very* easy to get wrong!


## Component election
Some components are special and must be executed exclusively across the cluster.
Yet we want more then one copy of them running so if the primary process fails a copy can take over.

Distributed coordination is used to achieve this:

  * Each component that needs it attempts to acquire leadership.
  * If a leader exists the process does nothing and watches the leader in case it fails.
  * If a leader does not exist the process becomes the leader and starts performing its function.
  * Before acting, and within reason, the primary process should check if it is still primary.
    This is to make sure that connection issues to the coordinator do not lead to double primary.
    * A process based on a periodic loop can check its status at the start of each run.

The implementation details may very over time and based on backends (i.e, Consul vs Zookeeper).

### Uses for component election

  * Cluster discovery process (periodically discovers clusters and pushes tasks to workers).


## Exclusive tasks
Some tasks may be scheduled too frequently or otherwise enqueued too often.
While in general this is not a problem, some tasks with side effects may cause issues
when run in parallel on the same inputs.

For these cases, tasks that should not be run in parallel acquire a lock at the start.
If the lock is acquired, the task proceeds as normal.
If the lock is already taken by another executor, the task is dropped.

### Uses for exclusive tasks

  * Cluster state refresh tasks (exclusive per cluster).


## Overload mitigation
Some tasks and/or operations may be expensive for the system.
Worse, they may be expensive for the agents!

To avoid expensive features over-scheduling leading to agent node failures or denial of service
any feature this expensive is limited in frequency.
Once such task is processed (usually successfully, at time even on failure) a self-destruct lock
is created for the parameters of the expensive task/operation.
Any task that is executed while this self-destruct lock exists is dropped.

### Uses for overload mitigation

  * The cluster state refresh task may impose performance penalties on cluster nodes.
    To avoid over-refreshing, a quiet period is introduced using the above technique.
