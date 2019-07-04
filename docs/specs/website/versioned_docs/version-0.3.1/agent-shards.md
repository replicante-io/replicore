---
id: version-0.3.1-agent-shards
title: Shards Information
sidebar_label: Shards Information
original_id: agent-shards
---

<blockquote class="warning">

**Alpha state disclaimer**

The protocol defined below is in early development cycle
and is subject to (potentially breaking) change.

</blockquote>


<div class="rest">
  <div class="method get">GET</div>
  <div class="url get">/api/unstable/shards</div>
  <div class="desc get rtl">Returns detailed information about shards</div>
</div>

The following information MUST be included:

  * A list of `shards` on the node:
    * The shard `id`.
    * The `role` of the node for the shard.
    * The (optional) `commit_offset` of the shard on the node:
      * The commit offset `unit`.
      * The commit offset `value` (as a 64-bits integer).
    * The (optional) `lag` of a secondary from its primary:
      * The lag `unit`.
      * The lag `value` (as a 64-bits integer).

Example:
```json
{
  "shards": [{
    "id": "shard_id",
    "role": "SECONDARY",
    "commit_offset": {
      "unit": "seconds",
      "value": 67890
    },
    "lag": {
      "unit": "seconds",
      "value": 33
    }
  }]
}
```
