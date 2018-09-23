---
id: migration-backend
title: Insane in the migration brain (a store migration backend)
---

Provide a special store backend that wraps two other stores.
This would allow online migrations from one store to another.

Primary concerns are:

  * Re-sharding collections in the event of storage format changes (i.e, organisations are added).
  * Move to other technologies: SQL solutions like [Vitess](https://vitess.io/).


## Challenges

  * Obviously the complexity in guaranteeing no data loss or corruption.
  * How to deal with mostly read-only data?
    * Assumed a copy on write system, should it become a copy on read?
    * Iterate through data and insert if missing?
