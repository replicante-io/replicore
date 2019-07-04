---
id: version-0.3.1-kafka-consumer
title: Rewrite Kafka Consumer
sidebar_label: Rewrite Kafka Consumer
original_id: kafka-consumer
---

Current consumer is extremely complex because it needs to deal with multiple threads.
It is also problematic because each thread needs all connections to kafka (all brokers or just some?).

One possibility is to trade complexity with higher task replay chances:

  * Only one consumer for tasks (and probably one for retries) exists per process.
  * It sits in a background thread that performs "bookkeeping" around which tasks have been acked and which are not.
  * This thread commits offsets to kafka as soon as all tasks up-to and including that offset is acked.
  * Worker threads requests tasks over a `bounded(0)` channel.
  * Acks and control messages are sent back over a different control channel.
  * When an offset commit fails it is retried a few times.
  * If the offset cannot be commited after a while the client is closed and bookkeeping reset to start consuming again.

## Downside
This design **should** be simpler, as long as bookkeeping does not prevent that,
and allows threads to scale without overloading the brokers with connections.

The cost is the reply of many tasks if one client fails in the presence of a slow tasks:
Other tasks on the same topic can be dispatched and processed but not acked with a slow task with "lower"
offset takes the entire client out.

As these cases should be rate, commits are retried so transient failures should not cause this,
the benefits may outweigh the costs.


## Why wait?

  * An implementation currently exists and works.
  * Should not optimise while essential features do not exist.
  * This will cause problems when scaling only, until then it is safe to wait.
