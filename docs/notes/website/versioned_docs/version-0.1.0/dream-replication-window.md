---
id: version-0.1.0-replication-window
title: Replication window metrics and re-sync times
original_id: replication-window
---

Intorduce some form of "oldest commit offset that can be replayed" metric for shards.
That, combined with the existing commit offset, can be used to determine:

  * How much can a node resync incrementally?
  * Looking at the speed of commit (can be computed by core), how much time do nodes get?

This assumes software would always perform incremental sync and that is not always true.
It may be best to not expose a minimum offset for such software (suspect most of them would not
have such a metric at all) and consider software that does not expose such metric to be one
that does not replay when a node re-joins.
