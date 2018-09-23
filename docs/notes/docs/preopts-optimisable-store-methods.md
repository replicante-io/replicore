---
id: optimisable-store-methods
title: Optimisable store methods
---

The current store API is very simple and potentially inefficient.
As the system becomes more complex and needs to support larger datasets this could become a bottle neck.

If the store API as it is now becomes too limited change things as follows:

  1. Required trait methods to iterate over TYPE items (all shards in cluster, all nodes in cluster, ...).
  2. Optional trait methods for expensive aggregations:
    1. Default trait uses the iterators to implement a naive approach.
    2. Implementation logic is kept outside of store crate for clean interface design?
    3. Stores that desire so can implement optimised version of default methods.

## Downside: code complexity

  * Where should default trait be?
  * If logic is to complex to fully optimise but can be partially optimised what should be done?
