---
id: version-0.3.0-scaling
title: Overview
sidebar_label: Overview
original_id: scaling
---

Replicante Core is designed as a distributed system meant to scale with user demand.

<blockquote class="info">

Scaling is an advanced topic that requires time and effort put into.

To achieve optimal value for money when considering the size of the cluster and number of tasks,
a degree of familiarity with each replicante component is required.

</blockquote>

<blockquote class="warning">

Replicante Core is at the early stages of development.

While scaling is a core feature of the platform, the limitations and requirements are not yet known.
Evolution of the system is likely to lead to changes in the scaling needs and configurations.

</blockquote>


## Replicante Core
Replicante Core stores its state out of process and using existing technologies
designed to scale state storage (databases, messaging systems, etcetera ...).
Process coordination also ensures that exclusive operations are performed safely
regardless of the number of processes running.

As a result Replicante Core processes themselves are stateless and can generally
be scaled by increasing the number of processes running.

The desired number of processes depends on the user's
[deployment configuration](admin-flexible-deployment.md) and their needs from the cluster.

Signals of the need to scale vary for each component.
The list below provides suggestions of what to look at for each component.

  * API components (this includes `components.grafana` and `components.webui`):
      look at the number of HTTP requests and their duration.
      Long running HTTP requests are an indication that something is not well.
      If other components and the datastores are healthy, long running HTTP requests may indicate
      a need to scale the API components.
  * Coordination/scheduling components (this includes `components.discovery`):
      these components only need a single instance running at any given time.
      To ensure all functionality remains available more then one instance of each service
      should be deployed so if the active instance fails another can take its place.
      Running 3 instances of each component should be the best form most situations.
  * Event consumers:
      event streams backup (rate of incoming events is higher then events processing rate).
      These components are similar to tasks (below) but must process events in order.
      Scaling the number of event consumers is as easy as running more instances.
      The complication may be with scaling the [streaming platform](scaling-streams.md).
  * Task workers (`components.workers`):
      task queue backup (rate of incoming tasks is higher then task processing rate).
      Scaling the number of task workers is as easy as running more instances.
      If scaling the worker instances is not enough users may need to scale the
      [task queues system](scaling-tasks.md).


## External systems
The more complex aspect of scaling tends to be at the state layer.

In most cases this means that the documentation of the dependencies will be the primary
source of information but some replicante-specific details are presented in these pages:

  * [Primary Store](scaling-store.md): where the current state of the system is stored.
  * [Tasks](scaling-tasks.md): for asynchronously processing data and performing tasks.
  * [Streams](scaling-streams.md): for ordered events occurring across the system.
  * [Coordinator](scaling-coordinator.md): for all processes to agree on work being done.
