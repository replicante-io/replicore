# Shards information
{% hint style="working" %}
**Alpha state disclaimer**

The protocol defined below is in early development cycle
and is subject to (potentially breaking) change.
{% endhint %}


{% rest %}
  {% get "/api/v1/shards" %}
    Returns detailed information about shards
{% endrest %}
{% method %}
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

{% common %}
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
{% endmethod %}
