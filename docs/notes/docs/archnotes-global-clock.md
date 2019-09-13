---
id: archnotes-global-clock
title: Global Clock
---

Time in distributed system is complex.  
There is no way to guarantee that all nodes will have an exact clock across all of them.

Therefore relying on an exact shared time for correctness leads to incorrect systems.

On the other hand it has been very painful and limiting to not introduce time at all:

  * Retries are based on time.
    Even now task retires use absolute times (from kafka) to decide if a task has to be retired.
  * Timeouts are based on an "operation start" sort of time.
    This is fine if the operation is within a single process but that only works for short times
    and does not survive process crashing or restarting.
  * User-oriented time data, such as `Event`s timestamps is essential.
    So far information about time of last sync and similar is missing because it requires time.

It should be clear at this point that some concept of time is needed but a global clock is not an option.
Each Replicante process uses the UTC time reported by the server it is running on assuming that
time on all other servers in the custer is "accurate enough".

What "accurate enough" means exactly depends on the operation being performed but sub-second
precision should NEVER be required for the system to function correctly.
On the other hand there is no guarantee the system works correctly in case the
[clock skew](https://en.wikipedia.org/wiki/Clock_skew) exceeds several minutes.

Final notes:

  * All generted times are in UTC and all received times expected to be in UTC as well.
    Replicante uses the [chrono](https://docs.rs/chrono/) library, which serialises values in
    [RFC 3339](https://tools.ietf.org/html/rfc3339) by default.
    All APIs MUST expose times in the same well known format.
    User interfaces (WebUI and `replictl`) can convert times as needed before showing.
  * Precision needs vary by task but should never be sub-second.
    Features that rely on higher than average time precision should document this.  
    **NOTE**: setups where time is precise enough for most tasks but not for high precision
    tasks the system may appear to work overall while some fetures actually do not.
  * The expectation is that most setups will be centrally managed by the same group
    therefore making reliance on time a more realistic option than the open internet.


<blockquote class="info">

For practical purposes all the above just means means that an
[NTP](https://en.wikipedia.org/wiki/Network_Time_Protocol) agent should be
running on all Replicante servers, including dependencies and monitored datastores.

</blockquote>
