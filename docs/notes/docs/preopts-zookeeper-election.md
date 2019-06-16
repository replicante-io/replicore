---
id: zookeeper-elections
title: Rewrite Zookeeper Elections
sidebar_label: Rewrite Zookeeper Elections
---

The current implementation of elections watches all election children (secondaries).
If a non-primary process terminates, every other node will be notified and perform reads
approximately at the same time and could overload zookeeper.

The official docs suggest a better implementation that I should aim for: https://zookeeper.apache.org/doc/r3.5.5/recipes.html#sc_leaderElection


## Why wait?

  * An implementation currently exists and works.
  * Should not optimise while essential features do not exist.
  * This will cause problems when scaling only, until then it is safe to wait.
