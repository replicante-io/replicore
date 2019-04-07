---
id: api-introspection
title: Introspection
sidebar_label: Introspection
---

The endpoints in this section provides information about the current state of the system.


<div class="rest">
  <div class="method get">GET</div>
  <div class="url get">/api/unstable/introspect/metrics</div>
  <div class="desc get rtl"></div>
</div>

Prometheus-style metrics exposition.

```text
# HELP replicore_discovery_fetch_errors Number of errors during agent discovery
# TYPE replicore_discovery_fetch_errors counter
replicore_discovery_fetch_errors 2
# HELP replicore_discovery_loops Number of discovery runs started
# TYPE replicore_discovery_loops counter
replicore_discovery_loops 1
```

<div class="rest">
  <div class="method get">GET</div>
  <div class="url get">/api/unstable/introspect/threads</div>
  <div class="desc get rtl"></div>
</div>

Show a list of threads and what they are currently working on.

<blockquote class="warning">

**The list is not complete**.  
The information is obtained from application-level instrumentation and not from the OS layer.
Threads that are created by external libraries used in the code will not register themselves
into this instrumentation.

</blockquote>

```json
{
  "threads": [{
    "activity": "(idle) election status: Primary",
    "name": "r:c:discovery",
    "short_name": "replicore:component:discovery"
  }, {
    "activity": "(idle) election status: Primary",
    "name": "r:coordinator:zoo:cleaner",
    "short_name": "replicore:coordinator:zookeeper:cleaner"
  }, {
    "activity": "running https://github.com/iron/iron HTTP server",
    "name": "r:i:api",
    "short_name": "replicore:interface:api"
  }, {
    "activity": "waiting for tasks to process",
    "name": "r:t:worker:0",
    "short_name": "replicore:tasks:worker:0"
  }],
  "warning": [
    "This list is NOT provided from an OS-layer instrumentation.",
    "As such, some threads may not be reported in this list."
  ]
}
```


<div class="rest">
  <div class="method get">GET</div>
  <div class="url get">/api/unstable/introspect/version</div>
  <div class="desc get rtl"></div>
</div>

Return version information about the running code:

  * `commit`: the git commit the code was build from.
  * `taint`: indicates if the code was changed compared to the commit.
  * `version`: friendly semantic versioning string.

```json
{
	"commit": "bbe5ddf4b62608974a35335014b854a650e72f7c",
	"taint": "working directory tainted",
	"version": "0.1.0"
}
```
