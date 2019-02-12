---
id: scaling-tasks
title: Task Queues
sidebar_label: Task Queues
---

Scaling the task queues is generally left to the queueing system itself.
Available options vary based on the selected backend.

Replicante works on the assumption that task queues do not provide any ordering guarantees
on the dispatching of tasks (newly scheduled tasks may be processed before tasks that were
scheduled earlier).

This assumption usually makes it easy to scale queueing systems.


## Kafka
[Kafka](https://kafka.apache.org/)'s scaling is based on the idea of
[topic partitions](https://kafka.apache.org/documentation/#intro_topics).

Replicante workers all share the same consumer group and consume messages from different topics,
one for each "kind" of task in the system.
This means that for each task "kind" replicante can process tasks in parallel up to the **lowest** of:

  * Number of replicante processes with the relevant `task_workers` enabled multiplied by `tasks.threads_count`.
  * Number of partitions in the topic for the task "kind".

To scale processing of tasks for a specific "kind" the lowest of the above factors (or both)
needs to be raised to match the needed concurrency level.

As mentioned above, Replicante does not rely on ordering for task processing so scaling
the number of partitions in each topic can simply be done (increasing a topic partitions
count usually disrupt ordering).
