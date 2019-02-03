---
id: rate-limits
title: Rate Limits
sidebar_label: Rate Limits
---

Rate limits may take different forms in different contexts but share the same goal:
limit, and ideally avoid, performance degradation and system abuse.


## API Rate Limits
API endpoints can do some serious damage if abused.
Rate limits are required to ensure that endpoints are not accidentally or intentionally used to cause harm.


## Overload mitigation
Some tasks and/or operations may be expensive for the system.
Worse, they may be expensive for the agents!

To avoid expensive features over-scheduling leading to agent node failures or denial of service
any feature this expensive is limited in frequency.
Once such task is processed (usually successfully, at time even on failure) a self-destruct lock
is created for the parameters of the expensive task/operation.
Any task that is executed while this self-destruct lock exists is dropped.

### Uses for overload mitigation

  * The cluster state refresh task may impose performance penalties on cluster nodes.
    To avoid over-refreshing, a quiet period is introduced using the above technique.
