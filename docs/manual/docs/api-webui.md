---
id: api-webui
title: WebUI
sidebar_label: WebUI
---

The WebUI is an external project to visualise and control clusters and automation
managed by Replicante Core.

These endpoints are not designed for use outside of the WebUI project.

<blockquote class="danger">

The WebUI project currently relies exclusively on this Replicante Core API but that is very
likely to change in the future with the introduction of a dedicated out-of-process WebUI server.

In that eventuality these endpoints are likely to be removed entirely.

</blockquote>


<div class="rest">
  <div class="method get">GET</div>
  <div class="url get">/api/unstable/webui/cluster/:cluster/discovery</div>
  <div class="desc get rtl"></div>
</div>

Return the discovery record for the specified `:cluster` ID.


<div class="rest">
  <div class="method get">GET</div>
  <div class="url get">/api/unstable/webui/cluster/:cluster/meta</div>
  <div class="desc get rtl"></div>
</div>

Return metadata for the specified `:cluster` ID.


<div class="rest">
  <div class="method get">GET</div>
  <div class="url get">/api/unstable/webui/clusters/find[/:query]</div>
  <div class="desc get rtl"></div>
</div>

List cluster IDs matching the given `:query`, which defaults to the empty string.  
Clusters match the query if their name or ID includes the `:query` string.


<div class="rest">
  <div class="method get">GET</div>
  <div class="url get">/api/unstable/webui/clusters/top</div>
  <div class="desc get rtl"></div>
</div>

Return the ID of the 10 clusters with the gratest number of nodes.


<div class="rest">
  <div class="method get">GET</div>
  <div class="url get">/api/unstable/webui/events</div>
  <div class="desc get rtl"></div>
</div>

Return the latest 100 events across the entire system.
