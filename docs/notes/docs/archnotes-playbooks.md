---
id: archnotes-playbooks
title: Playbooks
---

<blockquote class="warning">

**Playbooks are not a thing yet**

They will provide a way to automatically schedule actions to perform orchestrated
higher-lever operations across an entire cluster.
Writing about playbooks as if they were already here makes the entire system
(agents and core) easier to think about and design.

Aspects related to Playbooks and how they work are likely to change significantly.

</blockquote>

Playbooks are data structures defined in code and encoded in the primary store.
Playbooks are imported in the primary store through the API, usually loaded from a YAML file by `replictl`.
When stored in the DB, playbooks should also allow for some extra metadata such as a version or VCS commit ID or link.

When execution of a playbook starts, a playbook run record is created in the primary store
to track the overall state of the playbook and determine progression.
The full playbook is also copied in the playbook run record so that concurrent changes to the
playbook definition do not impact any already started run.

On top of state information for the playbook as a whole playbook runs also have to store
some per-agent state, traking details such as action result and "batch execution" style state.
Agent state also identifies nodes that are added or removed from the cluster while the playbook
was executing so playbooks can deal with these cases as appropriate.

This per-agent state is stored in a dedicated collection on the primary store:

  * When the playbook run starts a per-agent record is created for each known node.
    The node's state is set as `PENDING` to indicate it needs processing.
  * While the playbook runs nodes that are added or removed transition to
    `ADDED` or `REMOVED` states to indicate they need special consideration.
  * When a playbook run finishes a `finish_ts` timestamp is added to all
    records in the primary store (per-agent as well as the "global" run state).
    TTLed indexes are configured so the primary store automatically cleans up state
    for finished playbook runs.
