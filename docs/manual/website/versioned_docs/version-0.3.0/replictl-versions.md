---
id: version-0.3.0-replictl-versions
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
replictl: 0.3.0 [7655d363749aaa3d5cabbf603d4233b6a69efd45; working directory tainted]
Replicante (statically determined): 0.3.0 [7655d363749aaa3d5cabbf603d4233b6a69efd45; working directory tainted]
Replicante (dynamically determined): 0.3.0 [7655d363749aaa3d5cabbf603d4233b6a69efd45; working directory tainted]
Coordinator: Zookeeper (version not reported)
Primary Store: MongoDB 4.0.10
Tasks Queue: Kafka (version not reported)
```

## Replicante versions
The command returns three replicante versions:

  * `replictl` is the same as `replictl --version`.
  * `Replicante (statically determined)` is the version of replicante compiled with `replictl`.
  * `Replicante (dynamically determined)` is the version of replicante detected over the HTTP API.
