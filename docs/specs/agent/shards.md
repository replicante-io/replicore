# Shards information
> **[warning] Alpha state disclaimer**
>
> The protocol defined below is in early development cycle
> and is subject to (potentially breaking) change.


{% rest %}
  {% get "/api/v1/shards" %}
    Returns detailed information about shards
{% endrest %}
{% method %}
The following information MUST be included:

  * A list of all `shards` on the node:
    * The shard `id`.
    * The `role` of the node for the shard.
    * The `lag` (in seconds) of a secondary from its primary.
    * The UNIX timestamp (in seconds) of the last operation on the shard.

{% common %}
Example:
```json
{
  "shards": [{
    "id": "shard_id",
    "role": "SECONDARY",
    "lag": 12345,
    "last_op": 67890
  }]
}
```
{% endmethod %}
