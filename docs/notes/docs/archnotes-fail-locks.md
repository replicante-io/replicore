---
id: archnotes-fail-locks
title: Failure Modes: Locks
sidebar_label: Failure Modes: Locks
---

The behaviour of the system in the event of loosing a lock varies based on the operation
being performed while the lock is lost.


## Cluster state refresh
The refresh process is multi-phase:

  * Fetch nodes: lock is checked before each node is updated.
  * Data aggregation: lock is checked before updating.

### When a lock is lost during fetch
Each node is processed independently so nodes that have been updated before the lock was
lost will have their state updated correctly.

If the lock is ever lost between a check and an update, and another process is able to
update the state in the meantime, the newest update would be overwritten and the events
generated may be incorrect.

A possible solution could be to introduce a version or state hash field and perform
the update only if the current hash matches the expected one.

**The distributed nature of distributed system views means that minor
inconsistencies have to be expected**.

### When a lock is lost during aggregation
There is a possibility that lock is lost between the check before writing
the new updated data and the write itself.

In such case, the aggregate data would be stale and in need of regeneration by a cluster refresh.
