---
id: version-0.1.0-concurrent-agent-fetch
title: Concurrent Agent Fetches
original_id: concurrent-agent-fetch
---

Refreshing cluster nodes is performed in sequence, one node at a time.
The impact of it is limited to the cluster itself as clusters are processed in parallel.

For small clusters this is fine but as the number of nodes in a
cluster grows the performance penalties grow as well.
For now this is not a problem (mostly because nobody is using replicante yet) but
when managed clusters grow in size nodes should be refreshed in parallel.

## Complexity

  * A task worker would in turn use some sort of thread pool to perform its task.
  * At the time of writing the task-based fetcher does not even exist.
  * Ultimate performance for this part of processing could be achieved with futures/tokio.
    That adds quite a lot of complexity so maybe should be a third step?
