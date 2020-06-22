# Interfaces rewrite
Status: BLOCKED (async-rewrite; premature)
Reason:
  * The task scheduling interface is poor.
  * MAINLY: consumer is extremely complex and bugs out in playgrounds (worker thread deadlock).
Blocking something: NO


## Task
Re-write the tasks subsystem:

  * To provide a better API to schedule and consume tasks.
  * Implemented with simpler and cleaner code.
  * Avoid deadlocks in task workers
    * This has been seen often in playground mode.
    * Unclear what the actual cause is or what happens.
    * Seems to be linked to the consume never being polled.
    * Rewrite to be a fully async API may be simplest solution to this bit.


## Ideas
  * Trade consumer complexity with higher re-executie chances.
  * Only one consumer for tasks exists per process.
  * Perform "bookkeeping" around which tasks have been acked and which are not to handle concurrent executions.
  * On hard failures (can't commit offsets too many times):
    * Drop the consumer and reset bookkeeping.
    * Re-create the consumer.
    * Make sure any "ack" from ongoing tasks that were dropped are ignored.
    * Resume consuming tasks as if the process just started.
  * Use MongoDB to store tasks pending retry?
    * Instead of having two clients, one which is never/rarely polled.
    * Tasks are saved to mongo instead of pushed to a parallel queue.
      * Will likely need to support all stores supported as primary stores.
    * One process is responsible for "moving" tasks from mongo back to their queues.
      * This probably requieres access to the coordinator to pick someone responsible for it and replace the one that crashes.

## Downside
This design **should** be simpler, as long as bookkeeping does not prevent that, and allows threads to scale without overloading the brokers with connections.

The cost is the reply of many tasks is one client fails in the presence of a slow tasks (other tasks on the same topic can be dispatched and processed but not acked).
As these cases should be rare, comimts are retried so transient failures should not cause this, the benefits may outweigh the costs.
