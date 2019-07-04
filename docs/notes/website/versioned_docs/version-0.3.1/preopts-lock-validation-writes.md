---
id: version-0.3.1-lock-validation-writes
title: Lock validation before writes
original_id: lock-validation-writes
---

One of the biggest problems with coordination in distributed systems is dealing with isolation
from the distributed coordination system.

Several reasons may cause isolation and the ideal response may be different.
Unfortunately, some reasons look the same to the application but may be very different:

  * *The application is partitioned away from the coordinator*.
    The coordinator is still running but thinks we are not so it will release resources
    we think are currently entitled to have.
    This may lead to mutual exclusion violations for locks, multiple primaries, etcetera.
  * *The coordination service is down*.
    In this case the application will still be running but fail to check in with the coordinator
    and choose to relinquish control of acquired resources.
    This would lead to a spontaneous denial of services: the servers could provide whichever
    service it is supposed to but because they can't decide which one of them should nobody does.
  * *An application server is partitioned with a coordinator server*.
    The result here depends mostly on what the coordination system does.
    If the coordination system detects it is unable to provide its services to the application
    we likely fallback to the case where the application thinks the coordinator is down.
    If the coordination system ignores the fact that it is in the minority ... you should stop using it!

Replicante chooses consistency over availability: if the coordinator is not responsive, application
processes will assume they have no right to the resources they have and stop working.

The problem is the application process can't ensure the lock is held and the coordination works before
a write operation is performed.
This is because between the check and the write the lock may be lost for a number of reasons.
At the same time the code complexity grows fast and performance of both application and coordinator
decrease when the process attempts to ensure it holds the lock very often.

Replicante checks it holds locks before operations are performed and, for long held locks,
periodically in reasonable places (at the beginning and/or end of tasks).
This means the window of opportunity for the lock to be though as held by the application
but not by the coordinator is limited but still large.

Replicante could check its locks before every write operation to reduce that window
of opportunity to a write operation alone.


## Why wait?

  * This has not caused issues yet.
  * There is no definitive solutions, just ways to make it slightly better.
