# The Agent Protocol
> **[warning] Alpha state disclaimer**
>
> The protocol defined below is in early development cycle
> and is subject to (potentially breaking) change.
>
> The protocol is expected to change as it is used for implementations.
> In particular, the endpoint organization may be reviewed.

The Agents interface is a JSON encoded HTTP API.  
The API is versioned so that breaking changes can be rolled out gradually
with version compatibility "windows".


## `GET /api/v1/info`
Returns information about the agent itself.
The following information MUST be included:

  * `datastore` information:
    * `kind`: datastore software managed by the agent.
    * `version`: the datastore version.
  * `agent` information:
    * `version` information:
      * Version `number`: the [SemVer](https://semver.org/) agent version.
      * Version control `checkout`: VCS ID of the code that was build.
      * Version control `taint` status: indicates uncommitted changes to the code.

Example:
```json
{
  "datastore": {
    "kind": "MongoDB",
    "version": "3.4.5"
  },
  "agent": {
    "version": {
      "number": "0.1.0",
      "checkout": "11a24d9c3940f60e527c571680d64e80e0889abe",
      "taint": "not tainted"
    }
  }
}
```


## `GET /api/v1/status`
Returns the detailed status of the node, clustering status, sharding information.
The following information MUST be included:

  * A list of all `shards` on the node:
    * The shard `id`.
    * The `role` of the node for the shard.
    * The `lag` (in seconds) of a secondary from its primary.
    * The UNIX timestamp (in seconds) of the last operation on the shard.

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
