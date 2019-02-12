---
id: version-0.1.0-multi-cache
title: Multi-cache backend
original_id: multi-cache
---

Replicante will likely introduce a dependency on caching solutions at some point.
At first, a single [redis](https://redis.io/) instance, with replication of course, will be supported.

If a single cache instance becomes too large of a burden or causes too much damage when it becomes
unavailable, introduce a special cache backend to shard cached data:

  1. Namespace cache keys (if not already done).
  2. Add a new cache backend "multi-cache".
  3. It is configured with multiple actual caches (redises).
  4. Introduce some `namespace -> cache` mapping method (configuration file? Consistent hashing?)
  5. Multi-Cache backend transparently sends requests to the desired instance.
