---
id: version-0.3.1-from-discovery-to-events
title: From agent discovery to cluster events
original_id: from-discovery-to-events
---

## Background
The end goal of Replicante is to implement a specialised automation solution
to ensure no harm comes to your data as a result of automation.

At the core of this automation is a reactive engine: when a state change is detected actions
are taken to return the system to a desired state.

The idea of basing automation on a reactive engine is nothing new:

  * It is an easy model to understand for humans (physics is based on the action-reaction paradigm).
  * It is easy enough to implement (at least compared to other options).
  * It allows us to focus on the triggers of an event and the consequences of an action without
    having to look at the entire history of the full system.

The idea of a reactive engine may seem counter-intuitive.
After all we want to avoid issues, not just resolve them.
But this is easy to solve by reacting to events indicating an issue is on the way before reacting
to events indicating the issue has happended.

For example: we don't have to wait for a TLS certificate to expire before we start replacing it,
we can start the replacement process X days before the expiry time.
Our system just needs to emit an event to inform us it is time to start replacement.

Finally, events can tell us what is happening to our system and what we need to change
but they also tell us what our actions on the system lead to.


## Overview
![Overview: from discovery to events](assets/agent-events.png)

  1. When the system start, and periodically after that, clusters are discovered from one or more registers.
  2. A cluster status refresh task is queued up for for each cluster discovery record.
  3. When the cluster status refresh task is picked up, core refreshes the state of each node/agent.
  4. After all agents in the cluster have been refreshed (or fail to refresh) an aggregated view is built (and stored).
  5. Events are generated throught this process:
     * Node specific events are emitted while refreshing agent data.
     * Cluster level events are emitted while aggregating agents data.


## State aggregation
The newly fetched agents information is aggregated to generate an
[approximate cluster view](archnotes-cluster-view.md).
This cluster view is compared to the last known view to generate events describing changes
in the views of the system.

Because the cluster view is approximate (see [here](archnotes-cluster-view.md)) node events
are always based on reporting from the node themselves (we do not report a node as down if we
see it up, even if another node in the cluster think it is down).

Only cluster level events are generated off the top of this views.
Actions will have to check if the state of the system matches the expectations before they are applied.

### Avoid concurrent cluster observation
Because events are generated from differences in observed states, refreshing the state of
a node from multiple processes at once may lead to duplicate and/or missing events as well
as confused and inconsistent aggregations.

Distributed locks are used to ensure a cluster is refreshed by only one process at a time.
Any cluster refresh operation attempted while another operation is already running will be discarded.


## Why "discover" clusters?
The primary use case for Replicante is part of an automated, distributed, dynamic infrastructure that
scales from a small number of small cluster to a large number of large clusters.

It is assumed that managing a list of nodes is at best impractical, but may even be impossible
in combination with tools such as auto scaling groups and automated instance provisioners.

The idea of cluster discovery was inspired by [Prometheus](https://prometheus.io/).
Cluster discovery has several advantages:

  * A single source of truth as to which instances should exist in which clusters.
  * Automatic detection of node creation and retirement.
  * Much easier detection of agent issues (it is more difficult to tell why a push client is not pushing).
  * Agents are "checked" against some form of server inventory (server must impersonate agents to
    be polled, they can't just add themselves to the cluster and do whatever).


## Why Kafka?
[Kafka](https://kafka.apache.org/) itself isn't really the point,
the point is introspection and its implications on the ability to debug the system.

Kafka has the property that it keeps messages in its system for a configurable retention period
even if the message was consumed.
This feature will soon become a requirement for the events stream (it will be pushed to Kafka)
but even without that having access to the full history of messages can be helpful to understand
what is happening in the system and why.

In a distributed system where complexity runs high by nature,
any access to system introspection becomes an extremely valuable thing.

Additional task queues may be supported in the future but those that have this extra introspection
property will be favoured (at the time of writing this means
[NATS streaming](https://nats-io.github.io/docs/nats_streaming/intro.html)).
