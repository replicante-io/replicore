---
id: agent-actions
title: Actions API
sidebar_label: Actions API
---

<blockquote class="warning">

**Alpha state disclaimer**

The protocol defined below is in early development cycle
and is subject to (potentially breaking) change.

</blockquote>


<div class="rest">
  <div class="method get">GET</div>
  <div class="url get">/api/unstable/actions/finished</div>
  <div class="desc get rtl">Returns a list of finished actions</div>
</div>

Actions MUST be listed in order from newest to oldest.

When an action is finished, the agent will never change its state.

Finished actions should be cleaned up periodically to prevent this list
from growing too large and the agent state from taking over the node.

A list of finished actions MUST include:

  * The action ID.
  * The final action state.

And it MAY include:

  * The action kind.

Example:
```json
[
    {
        "kind": "replicante.store.stop",
        "id": "703824bf-2c16-44f5-b4da-b21688c57043",
        "state": "DONE"
    },
    {
        "kind": "replicante.store.stop",
        "id": "f4fdda3f-3130-474b-b22c-66c6824a5d89",
        "state": "DONE"
    },
    {
        "kind": "replicante.store.stop",
        "id": "191cc19b-2dee-4013-b908-29c7985f79ac",
        "state": "DONE"
    }
]
```


<div class="rest">
  <div class="method get">GET</div>
  <div class="url get">/api/unstable/actions/queue</div>
  <div class="desc get rtl">Returns a list of currently running or queued actions</div>
</div>

Actions MUST be listed in order from oldest to newest.

The format of elements in this list is the same as the one of items
returned by `/api/unstable/actions/finished`.

Example:
```json
[
    {
        "kind": "replicante.store.stop",
        "id": "703824bf-2c16-44f5-b4da-b21688c57043",
        "state": "RUNNING"
    },
    {
        "kind": "replicante.store.stop",
        "id": "f4fdda3f-3130-474b-b22c-66c6824a5d89",
        "state": "NEW"
    },
    {
        "kind": "replicante.store.stop",
        "id": "191cc19b-2dee-4013-b908-29c7985f79ac",
        "state": "NEW"
    }
]
```


<div class="rest">
  <div class="method get">GET</div>
  <div class="url get">/api/unstable/actions/info/:id</div>
  <div class="desc get rtl">Returns an action details as well as its state history</div>
</div>

The following parameters are REQUIRED in the URL:

  * `:id`: the ID of the action to lookup.

The response will include the following information:

  * `action`: the full action model as described in the [protocol section](agent-intro.md#actions).
  * `history`: array of action transition events:
    * `action_id`: (optional) ID of the action that transition.
                   If set, this MUST be the same as `action.id`.
    * `timestamp`: the (agent) time the action entered the state.
    * `state`: the state that was reached.
    * `state_payload`: optional JSON value defined by the action at the time of transition.

Example:
```json
{
    "action": {
        "kind": "replicante.service.gracefulrestart",
        "created_ts": "2019-08-30T20:40:24Z",
        "finished_ts": "2019-08-30T20:40:37Z",
        "headers": {},
        "id": "308fb8bc-79a1-49d9-bf71-1191d7d6c5d2",
        "requester": "API",
        "args": {},
        "state": "DONE",
        "state_payload": {
            "payload": {
                "attempt": 0,
                "message": "the service is running",
                "pid": "11634"
            },
            "stage": 2,
            "state": "DONE"
        }
    },
    "history": [
        {
            "action_id": "308fb8bc-79a1-49d9-bf71-1191d7d6c5d2",
            "timestamp": "2019-08-30T20:40:37Z",
            "state": "DONE",
            "state_payload": {
                "payload": {
                    "attempt": 0,
                    "message": "the service is running",
                    "pid": "11634"
                },
                "stage": 2,
                "state": "DONE"
            }
        },
        {
            "action_id": "308fb8bc-79a1-49d9-bf71-1191d7d6c5d2",
            "timestamp": "2019-08-30T20:40:33Z",
            "state": "RUNNING",
            "state_payload": {
                "payload": {
                    "attempt": 0,
                    "message": "the service is not running",
                    "pid": null
                },
                "stage": 1,
                "state": "DONE"
            }
        },
        {
            "action_id": "308fb8bc-79a1-49d9-bf71-1191d7d6c5d2",
            "timestamp": "2019-08-30T20:40:30Z",
            "state": "RUNNING",
            "state_payload": {
                "payload": {
                    "message": "Err(OperationError(\"No servers available for the provided ReadPreference.\"))"
                },
                "stage": 0,
                "state": "DONE"
            }
        },
        {
            "action_id": "308fb8bc-79a1-49d9-bf71-1191d7d6c5d2",
            "timestamp": "2019-08-30T20:40:24Z",
            "state": "NEW",
            "state_payload": null
        }
    ]
}
```


<div class="rest">
  <div class="method post">POST</div>
  <div class="url post">/api/unstable/actions/schedule/:kind</div>
  <div class="desc post rtl">Request the scheduling of a new action</div>
</div>

The following parameters are REQUIRED in the URL:

  * `:kind`: the ID of the action to lookup.

A JSON body is REQUIRED for this endpoint:

  * The entiry JSON body is passed as arguments to the request.

The agent is REQUIRED to validate the agruments passed to the request.
If the provided arguments are incompatible to what the action `:kind` expects
the endpoint MUST return an HTTP 400 error to the caller.

The response will include the following information:

  * `id`: unique ID of the newly scheduled action.

Example request:
```json
{}
```

Example response:
```json
{
    "id": "308fb8bc-79a1-49d9-bf71-1191d7d6c5d2"
}
```
