---
id: version-0.3.0-api-introspection
title: Introspection
sidebar_label: Introspection
original_id: api-introspection
---

The endpoints in this section provides information about the current state of the system.


<div class="rest">
  <div class="method get">GET</div>
  <div class="url get">/api/unstable/introspect/health</div>
  <div class="desc get rtl"></div>
</div>

Expose the results of internal dependencies health checks.

The endpoint always returns a 200 HTTP status code to indicate that the process
itself is running, even if dependencies are degraded or have failed.

```json
{
  "age_secs": 1,
  "results": {
    "coordination": {
      "status": "HEALTHY"
    },
    "store-primary": {
      "status": "HEALTHY"
    },
    "tasks-consumer": {
      "status": "HEALTHY"
    },
    "tasks-producer": {
      "status": "HEALTHY"
    }
  }
}
```

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
  <div class="url get">/api/unstable/introspect/self</div>
  <div class="desc get rtl"></div>
</div>

Reports the node's identity as registered in the coordinator.
The `extra` attributes are reported as specified in the node's config file.

```json
{
  "extra": {},
  "id": "4e7bff37f6b9e584dd213aadb33443f9"
}
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
  "threads": [
    {
      "activity": "(idle) waiting for the next healthchecks run",
      "name": "replicante:interface:healthchecks",
      "short_name": "r:i:healthchecks"
    },
    {
      "activity": "running https://github.com/iron/iron HTTP server",
      "name": "replicore:interface:api",
      "short_name": "r:i:api"
    },
    {
      "activity": "(idle) election status: Primary",
      "name": "replicore:service:coordinator:zookeeper:cleaner",
      "short_name": "r:s:coordinator:zoo:c"
    },
    {
      "activity": "(idle) waiting for tasks to process",
      "name": "replicore:service:tasks:worker:0",
      "short_name": "r:s:tasks:worker:0"
    },
    {
      "activity": "(idle) waiting for tasks to process",
      "name": "replicore:service:tasks:worker:1",
      "short_name": "r:s:tasks:worker:1"
    }
  ],
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
