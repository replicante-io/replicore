---
id: scaling-events
title: Event streams
sidebar_label: Event streams
---

Scaling the event streams is generally left to the streaming platform itself.
Available options vary based on the selected backend.

Replicante works on the assumption that event streams provide some level of ordering guarantees.
In particular ordering of events that share the same key MUST be guaranteed with respect to
each other while no ordering guarantees are expected of events that do not share keys.

This expectation usually makes it more difficult to scale events streams without downtime.


## Kafka
<blockquote class="warning">

Event streams are not yet implemented but are on their way.

The notes in this page are based on the expected design of the feature and will be
updated once the event streams are implemented to reflect the final result.

This notes are provided as guidelines for prospecting users for what to expect.

</blockquote>


[Kafka](https://kafka.apache.org/)'s scaling is based on the idea of
[topic partitions](https://kafka.apache.org/documentation/#intro_topics).

Events are streamed to a single topic and partitioned as follow:

  * Events that relate to a cluster are partitioned by cluster ID.
  * All other events are partitioned under the key `<system>`.

This has implications to scaling kafka topics by adding partitions:
it is difficult to add partitions while maintain ordering guarantees unless
**no writes occur and all events have been processed** prior to scaling partitions.

<blockquote class="danger">

In the initial implementation of event streams no support for changes to the partitions
count while the system is operation will be provided.

While future versions may provide for such operations they will still be complex.
You are advised to over-provision the number of partitions on event streams to limit,
or even avoid, the need to change the number of partitions while the system is operating.

</blockquote>
