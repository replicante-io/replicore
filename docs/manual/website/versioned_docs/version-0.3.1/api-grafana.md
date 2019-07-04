---
id: version-0.3.1-api-grafana
title: Grafana
sidebar_label: Grafana
original_id: api-grafana
---

Endpoints for use with Grafana integration.  
Not designed for use in other contexts.


<div class="rest">
  <div class="method get">GET</div>
  <div class="url get">/api/unstable/grafana</div>
  <div class="desc get rtl"></div>
</div>

Check endpoint that returns 200 used by the
[Simple JSON Datasource](https://grafana.com/plugins/grafana-simple-json-datasource)
grafana plugin.


<div class="rest">
  <div class="method post">POST</div>
  <div class="url post">/api/unstable/grafana/annotations</div>
  <div class="desc post rtl"></div>
</div>

Annotations query endpoint used by the
[Simple JSON Datasource](https://grafana.com/plugins/grafana-simple-json-datasource)
grafana plugin.

Filtering options can be passed as a JSON object to the `query` parameters.  
All filters are optional and include most events by default:

  * `cluster_id` (default: `null`): Only show events for this cluster.
  * `event`: (default: `null`): Only show events matching this event type.
  * `exclude_snapshots` (default: `true`): Exclude `SNAPSHOT_*` events from the results.
  * `exclude_system_events`: (default: `false`): Exclude system-wide events from the results.
  * `limit`: (default: `1000`): Limit the number of events returned, starting with oldest events.
