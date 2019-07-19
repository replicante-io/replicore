---
id: split-discovery
title: Split Discovery and Refresh
sidebar_label: Split Discovery and Refresh
---

The discovery component is currently driving cluster refreshes.
This means that for a cluster to be refreshed the entire discovery run has to work.

Slow discovery backends or backends with many clusters will impact cluster refresh
regularity and consistenty punish and vafour the same suspects.
It also means that cluster refreshes happen all at the same time and at the same
frequencies.

Splitting the two tasks would be more consistent and reliable and would
unlock a bunch of possible improvements to the system:

  * Per-cluster/org refresh rate.
  * Refreshes even when discovery has issues.
  * More advanced score based refresh priorities.
  * Sharded discovery/refresh runs.

## Possible implementation

  1. Make `cluster/fetcher` and `cluster/aggregator` use a provided `discovery` and NEVER fetch it from the store.
     * Initially fetch the previous state in the refresh task or discovery loop.
  2. Update discoveries to be stored as `previous: {}, current: {}` documents.
  3. The discovery component inserts/updates stored `discovery` records.
     * Discovery component no longer spawns refresh tasks.
  4. Introduce a refresher HA component:
     * Scan through all `discovery` records (priv & current), ordered by ID to make use of indexes and ensure no duplicate results (because MongoDB).
     * Spawn cluster refresh records from here.
  5. **Optional** support sharding discovery scan and/or cluster refresh:
     * Use pre-defined filters so pick a subset of records.
     * Ensure the search space is always fully covered.


## Why wait?

  * This is not needed now.
  * Significant changes needed: cluster discovery record may change while refresh is in progress.
