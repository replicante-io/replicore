---
id: admin-flexible-deployment
title: Flexible Deployment
sidebar_label: Flexible Deployment
---

Replicante Core comes as a single binary with configurable components.

Some components provide optional functionality.
The `grafana` component for example only provides the endpoints used for
[Grafana annotations](features-events.md#grafana-annotations) and can be disabled
if the functionality provided is not useful to the user.

By default, all components are enabled on all nodes.
The exception are components providing experimental or deprecated functionality.


## Uniform deployment
As a result of these defaults, Replicante Core runs as a *uniform deployment*:
each node is equivalent to all other nodes and capable of performing the same functions.

The main advantage of a uniform deployment is simplicity:

  * Each node is the same as all others.
  * The system is simpler to understand: no "who does what?" scenarios.

This simplicity comes at a cost:

  * One component miss-behaving on a node can impact all other components on the same node.
  * Expensive API requests can impact system functionality and system functionality can impact API requests.
  * Scaling the system is less granular: the bottleneck sets the "size" for all components.


## Dedicated API servers
Another very common deployment configuration is based on separating the API servers from
other components in the system.
This protects the system from expensive API requests and ensures that API requests are not
effected by expensive system operations.

To deploy dedicated API servers two "roles" are created by using different configurations
for each role.
Most options are the same across both roles so only the different parts are shown here.

For the "API server" role use the following configuration:
```yaml
components:
  _default: false  # Disable all components not explicitly listed.
  grafana: true  # Optionally enable the grafana annotations API too.
  webui: true
```

For the "system server" role, the one running all non-API components, use the following configuration:
```yaml
components:
  _default: true  # Enable all components not explicitly listed.
  grafana: false  # Disable grafana annotations API.
  webui: false  # Disable the WebUI API.
```

While this approach isolates API requests from all other system functionality
it does not address any of the other limitations of a uniform deployment.


## Dedicated everything
The approach of dedicated roles can be extended from API vs everything else
to the extreme of a role for each component.

It is time to mention that replicante also has a `workers` component that runs asynchronous tasks.
To pick which tasks a node should be working on the `task_workers.*` configuration options
can be used in a similar way to `components.*`.

To define a role for a component use the following configuration snippet:
```yaml
components:
  _default: false
  <COMPONENT>: true
```

For task worker roles use the following configuration snippet:
```yaml
components:
  _default: false
  workers: true

task_workers:
  _default: false  # Disable processing of tasks not explicitly enabled.
  <TASK_TYPE>: true
```

While such a diverse deployment requires a bit more effort to manage it does have several advantages:

  * Each component and/or task worker can scale independently as needed.
  * Components are isolated from each other (through the user's preferred process isolation solution).


## To each their own
From the configuration snippets above is should be clear that a spectrum of deployments is possible.
Users are free to configure different nodes to best fit their needs.

To ensure that all required components are configured and available somewhere in the cluster
replicante exposes metrics for which components are enabled on each node.

The following prometheus queries can aid operators monitor components and task workers across
all nodes in a Replicante Core cluster:

  * `replicore_components_enabled{type="required"}`: should be `>= 1` for each component.
  * `replicore_workers_enabled`: should be `>= 1` for each worker.

There is also a query to count optional components: `replicore_components_enabled{type="optional"}`.
Users that would like to make use of optional features should ensure that components providing
such features are available somewhere in the cluster.
