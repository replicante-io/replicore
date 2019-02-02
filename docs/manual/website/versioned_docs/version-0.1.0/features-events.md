---
id: version-0.1.0-features-events
title: Events
sidebar_label: Events
original_id: features-events
---

Before you can react to a change in the system you need to know that something has changed.

Replicante first task is to observe all nodes and generate events to reflect changes to nodes and clusters.
These events are internally used to drive features but are also recoded for the user to see.

Having access to historical events can provide valuable insight.
Increase error rates? Is/was the datastore down or in the middle of a failover?
Being able to correlate datastore events with application errors, performance issues,
or other unusual activity is key into improving your own code.


## WebUI history view
The simplest way to view the events history is to check out the `Events` page in the [WebUI](features-webui.md).


## Stream subscription
This feature will be available soon ...


## Grafana integration
This feature will be available soon ...
