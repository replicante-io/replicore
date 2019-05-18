# Error names
Status: **ONGOING**
Reason: Sentry integration currently picks up `ErrorKind` for everything.
Blocking something: NO


## Task
The `failure` crate supports an optional `name` method that sentry calls to pick an exception name.
As this is not implemented, it defaults to `Error` or `ErrorKind`, which does not help.
The error type for the `replicante` crate implements this so reported names are helpful.

The task is to update all crates to expose this information:

  * [ ] agent/client
  * [ ] agent/discovery
  * [ ] common/util/tracing
  * [ ] coordinator
  * [ ] data/aggregator
  * [ ] data/fetcher
  * [ ] data/store
  * [ ] replictl
  * [ ] streams/events
  * [ ] tasks
  * [x] replicante

  * [ ] agents: base
  * [ ] agents: kafka
  * [ ] agents: mongodb
  * [ ] agents: zookeeper
