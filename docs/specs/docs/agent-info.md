---
id: agent-info
title: Agent Information API
sidebar_label: Agent Information API
---

<blockquote class="warning">

**Alpha state disclaimer**

The protocol defined below is in early development cycle
and is subject to (potentially breaking) change.

</blockquote>


<div class="rest">
  <div class="method get">GET</div>
  <div class="url get">/api/unstable/info/agent</div>
  <div class="desc get rtl">Returns information about the agent itself</div>
</div>

The following agent information MUST be included:

  * `version` information:
    * Version `number`: the [SemVer](https://semver.org/) agent version.
    * Version control `checkout`: VCS ID of the agent code that is running.
    * Version control `taint` status: indicates uncommitted changes to the code.

Example:
```json
{
  "version": {
    "number": "0.1.0",
    "checkout": "11a24d9c3940f60e527c571680d64e80e0889abe",
    "taint": "not tainted"
  }
}
```


<div class="rest">
  <div class="method get">GET</div>
  <div class="url get">/api/unstable/info/datastore</div>
  <div class="desc get rtl">Returns information about the datastore</div>
</div>

The following datastore information MUST be included:

  * `cluster_id`: datastore determined cluster identifier.
  * `id`: datastore determined, cluster unique, node identifier.
  * `kind`: datastore software managed by the agent.
  * `version`: the [SemVer](https://semver.org/) datastore version.

The following datastore information MAY be included:

  * `cluster_display_name`:
    cluster display name to be used in place of the `cluster_id` in
    the UI and other user messages, if provided.

Example:
```json
{
  "cluster_display_name": "prod-data",
  "cluster_id": "replica-set-name",
  "id": "host.domain.com:27017",
  "kind": "MongoDB",
  "version": "3.4.5"
}
```
