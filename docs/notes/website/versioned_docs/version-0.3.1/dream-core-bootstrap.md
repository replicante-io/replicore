---
id: version-0.3.1-core-bootstrap
title: Replicante bootstrapping procedure
original_id: core-bootstrap
---

Allow one-command start of a Replicante instance that can be used to configure
all datastores and dependences for Replicante itself, then generate a configuration
file to be used for production instances.

May be just a guide but will likely require code changes/extra features.
Most likely, in-memory version of the dependencies should be made available so the process can
start with no additional complexity or impairment.

  * Consider the all-in-one process like jaeger does.
  * Probably introduce some `all_in_one` cargo feature for specialied "backends" that can be excluded from release builds.
  * Coordinator: in-memory mutexes, elections are all primaries.
  * Store: document-store version of sqlite so I don't need to implement my thing?
  * Tasks: use `crossbeam_channel`s to fake queues, drop on skip (and retry?).
  * Streams: same as tasks.
