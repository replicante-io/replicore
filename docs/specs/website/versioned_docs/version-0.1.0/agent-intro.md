---
id: version-0.1.0-agent
title: The Agent Protocol
sidebar_label: The Agent Protocol
original_id: agent
---

<blockquote class="warning">

**Alpha state disclaimer**

The protocol defined below is in early development cycle
and is subject to (potentially breaking) change.

</blockquote>

The Agents interface is a JSON encoded HTTP API.  
The API is versioned so that breaking changes can be rolled out gradually
with version compatibility "windows".


## Agent information
<div class="rest">
  <div class="method get">GET</div>
  <div class="url get">/api/v1/info/agent</div>
  <div class="desc get rtl">Returns information about the agent itself</div>

  <div class="method get">GET</div>
  <div class="url get">/api/v1/info/datastore</div>
  <div class="desc get rtl">Returns information about the datastore</div>
</div>

[Details about these endpoints](agent-info.md)


## Shards information
<div class="rest">
  <div class="method get">GET</div>
  <div class="url get">/api/v1/shards</div>
  <div class="desc get rtl">Returns detailed information about shards</div>
</div>

[Details about these endpoints](agent-shards.md)
