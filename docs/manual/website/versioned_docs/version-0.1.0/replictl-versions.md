---
id: version-0.1.0-replictl-versions
title: replictl versions
sidebar_label: replictl versions
original_id: replictl-versions
---

Collect version information about replicante and all configured services:

<blockquote class="info">

This command requires access to all configured services for the version report to be complete.

</blockquote>

```text
$ replictl versions
Jun 12 21:58:54.512 INFO Showing versions, version: b73969f8b19a17cd881f4e04741cdc764757b7ef, module: replictl::commands::versions
Jun 12 21:58:54.513 INFO Configuring MongoDB as storage validator, version: b73969f8b19a17cd881f4e04741cdc764757b7ef, module: replicante_data_store::backend::mongo
replictl: 0.1.0 [b73969f8b19a17cd881f4e04741cdc764757b7ef; index and working directory tainted]
Replicante (statically determined): 0.1.0 [f7a52185302d119cc1707b9d0a69bf29ac5b4633; working directory tainted]
Replicante (dynamically determined): 0.1.0 [f7a52185302d119cc1707b9d0a69bf29ac5b4633; working directory tainted]
Store: MongoDB 3.6.4
```

## Replicante versions
The command returns three replicante versions:

  * `replictl` is the same as `replictl --version`.
  * `Replicante (statically determined)` is the version of replicante compiled with `replictl`.
  * `Replicante (dynamically determined)` is the version of replicante detected over the HTTP api.
