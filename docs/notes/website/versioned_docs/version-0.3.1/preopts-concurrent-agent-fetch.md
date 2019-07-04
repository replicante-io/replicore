---
id: version-0.3.1-concurrent-agent-fetch
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
  * Ultimate performance for this part of processing could be achieved with futures/tokio.
    That adds quite a lot of complexity so maybe should be a third step?
    * It could be possible to run fetch processes in a scoped event loop.
    * The main application remains threaded and segregated.
    * Task workers are still dedicated threads too and most task can be sync.
    * Cluster Refresh tasks would be the only to use futures/tokio for async work.


## Why wait?

  * Replicante Core functionality is not yet product-complete.
  * Should not optimise for performance while core features are not implemented.
