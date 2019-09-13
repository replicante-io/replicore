---
id: archnotes-scheduling-actions
title: Scheduling Actions
---

Each agent can run a single action at any time and has a queue of actions waiting to be executed.
These actions can be scheduled either directly though the agent API or through Replicante Core.
Replicante Core can schedule actions on behalf of a user or automatically as part of a playbook.

<blockquote class="warning">

**Playbooks are not a thing yet**

They will provide a way to automatically schedule actions to perform orchestrated
higher-lever operations across an entire cluster.
Writing about the actions system assuming playbook are already here makes
the entire system (agents and core) easier to think about and design.

</blockquote>

Generally, an agent's actions queue should be short.
Replicante Core will not generate playbook actions until the playbook reaches the stage
where the actions are needed and manually scheduled actions are expected to be few.

## Actions and playbook progression
Actions progress when the agent reports that their state has changed.
Actions progress from the `NEW` state to a finished state like `DONE` or `FAILED`.

Playbooks progress when all actions in the current stage have finished.
Playbooks progress in a similar way to actions, moving along stages.

Both actions and playbooks progress checks are reactive: events have to trigger them.
This is opposed to proactive checks where the system would have to poll pending and running
actions and playbooks and check if they can progress.

Reactive checks are more efficient because resources are not spent checking
over and over for states that have not changed.
Reactive checks also make Replicante Core far simpler to implement.

Events that cause actions and playbook progression are emitted by the cluster state
refresh tasks when agent actions state changes or when pending actions 
To ensure users have an escape hatch in case the system fails to auto-detect
progress the API will allow the refresh the cluster task to `nudge` actions and playbooks.
