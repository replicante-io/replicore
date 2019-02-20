---
id: admin-tasks
title: Tasks subsystem
sidebar_label: Tasks subsystem
---

Replicante uses an asynchronous tasks subsystem to schedule operations.

The health of the tasks subsystem is critical to the correct functioning of the system.
Nodes configured to run the `workers` component are responsible for the execution of tasks.
The tasks each node processes are configured with the `task_workers.*` options.

<blockquote class="warning">

Be aware that if the task system becomes unhealthy other parts of the system may appear
to be functioning correctly but operations may not be performed or they may be delayed.

</blockquote>

Once at least one node is configured with the `workers` component enabled the `replicore_workers_enabled`
family of metrics reports how many worker nodes are enabled for each task worker.

Additionally the following metrics can be used to check the health of the workers:

  * `replicore_tasks_worker_received`: number of tasks each node has received for each task queue.
  * `replicore_tasks_ack_errors`: number of task acknowledgements that failed to be processed
                                  by the configured external task queue (i.e, kafka).
    * `{op="fail"}`: error acknowledging a failed task.
    * `{op="fail[skip]"}`: error acknowledging a task that was skipped because it failed too many times.
    * `{op="skip"}`: error acknowledging a skipped task (task the code decided not to process).
    * `{op="success"}`: error acknowledging a successfully processed task.
  * `replicore_tasks_ack_total`: number of task acknowledgements that were sent to the configured
                                 external task queue (i.e, kafka).
    * `{op="fail"}`: acknowledgement of a failed task.
    * `{op="fail[skip]"}`: acknowledgement of a task that was skipped because it failed too many times.
    * `{op="skip"}`: acknowledgement of a skipped task (task the code decided not to process).
    * `{op="success"}`: acknowledgement of a successfully processed task.

<blockquote class="info">

When monitoring the tasks subsystem, or the system in general,
always incorporate metrics reported by Replicante's dependencies.
This information may be helpful in spotting issues that would not
be clear through Replicante metrics only.

</blockquote>
