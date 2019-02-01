---
id: api-monitoring
title: Monitoring
sidebar_label: Monitoring
---

The endpoints in this section provides information about the current state of the system.


<div class="rest">
  <div class="method get">GET</div>
  <div class="url get">/api/v1/metrics</div>
  <div class="desc get rtl"></div>
</div>

Prometheus-style metrics exposition.

```text
# HELP replicante_discovery_fetch_errors Number of errors during agent discovery
# TYPE replicante_discovery_fetch_errors counter
replicante_discovery_fetch_errors 2
# HELP replicante_discovery_loops Number of discovery runs started
# TYPE replicante_discovery_loops counter
replicante_discovery_loops 1
```


<div class="rest">
  <div class="method get">GET</div>
  <div class="url get">/api/v1/version</div>
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
