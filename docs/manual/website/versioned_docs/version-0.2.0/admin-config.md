---
id: version-0.2.0-admin-config
title: Configuration
sidebar_label: Configuration
original_id: admin-config
---

Replicante provides a large set of configuration options with reasonable defaults.
This supports common use cases where only a handful of options need attention
as well as advanced setup where the user has the power to fine tune most details.


## Required configuration
Some options do not have reasonable defaults so users will have to set them explicitly:

  * [Agents discovery](features-discovery.md) (vastly depends on user needs).
  * Storage configuration (specifically: address of the DB server).


## Configuration options
All options are documented in the
[example configuration file](https://github.com/replicante-io/replicante/blob/master/replicante.example.yaml)
at the root of the repo, also shown below.

This file shows all options with their defaults and explains their meaning and available settings.
As mentioned above, common use cases should be able to ignore most options if users are so inclined.


<blockquote class="info">

In most cases, details of these options are documented in the features they influence.

Options that don't directly relate to a single feature are instead documented below the example file.

</blockquote>

```yaml
# The section below is for the API interface configuration.
api:
  # The network interface and port to bind the API server onto.
  #
  # By default, only bind to the loopback interface.
  # Production environments should place an HTTPS proxy in front of the API.
  bind: '127.0.0.1:16016'


# Components enabling configuration.
#
# For Replicante to function correctly ALL components need to be running
# (unless explicitly stated otherwise).
#
# By default, all required components are enabled on all nodes.
#
# While this allows the configuration to stay simple (all nodes in the cluster are identical),
# more advanced configuration options based around specialised nodes roles can be used to
# scale to larger clusters in a more efficient way (because roles can be scaled based on
# each role needs rather then scaling all roles based on the bottleneck).
#
# # Exceptions to the ALL components rule
# Some components are not required for essential operations.
# These include new features being beta-tested or features that you may not need.
#
# For example the `grafana` component can be used to enable grafana endpoints annotations.
# If these are not needed, the endpoint can be left disabled.
#
# # Advanced configurations
# Advanced configurations are aimed at large clusters or other specialised needs.
# By default, start with all components enabled on all nodes and move to roles if and when needed.
#
# ## Dedicated Web/API nodes
# The first thing to split aside are the API and Web nodes.
# API/Web nodes are the ones that receive requests from users and have less
# predictable resource requirements.
#
# Separating them from other components means that:
#
#   * Users can use all resources without background tasks impacting them.
#   * Having other components not impacted by user requests.
#
#     # For the API/Web nodes
#     components:
#       _default: false
#       grafana: true  # Optional
#       webui: true
#
#     # For all other nodes
#     components:
#       _default: true
#       grafana: false
#       webui: false
#
# ## Fully dedicated components
# If more control is needed, further roles can be defined by disabling components in the
# general case (named "all other nodes" in the example above) and enabling on their own
# in a new, specialised, role.
components:
  # Default status for all components that are not explicitly enabled/disabled.
  _default: true

  # Enable/disable agent discovery.
  discovery: null

  # Enable Grafana Annotations API endpoints (optional).
  grafana: null

  # Enable the WebUI API endpoints (optional).
  #
  # Without them, the nodejs WebUI will NOT work.
  webui: null

  # Enable the task workers component to process tasks.
  workers: null


# Distributed coordinator configuration options.
coordinator:
  # The distributed coordination system to use for nodes coordination.
  #
  # !!! DO NOT CHANGE AFTER INITIAL CONFIGURATION !!!
  # This option is to allow users to choose a supported software that best fits
  # their use and environment.
  #
  # Available options:
  #
  #   * 'zookeeper' (recommended)
  backend: 'zookeeper'

  # User specified key/value map attached to node IDs.
  #
  # This data is not used by the system and is provided to help users debug
  # and otherwise label nodes for whatever needs they may have.
  node_attributes: {}

  # Any backend-specific option is set here.
  # The available options vary from backend to backend and are documented below.
  #
  # Zookeeper options:
  options:
    # Zookeeper ensemble connection string.
    ensemble: 'localhost:2181/replicante'

    # Zookeeper background cleaner configuration.
    #
    # The background cleaner thread ensures that znodes no longer needed are removed
    # from zookeeper to keep the number of znodes to a minimum.
    cleanup:
      # Maximum number of nodes to delete in a single cleanup cycle.
      limit: 1000

      # Seconds to wait between cleanup cycles.
      interval: 3600

      # Number of cycles before this node will re-run an election, 0 to disable re-runs.
      #
      # Having the system re-run elections continuously ensures that failover procedures are
      # exercised constantly and not just in case of errors.
      # You do not want to discover that failover does not work when a primary fails
      # and nothing picks up after it.
      #
      # With the default interval, this default will cause a re-election about every 6 hours.
      term: 6

    # Zookeeper session timeout (in seconds).
    timeout: 10


# The section below is for agent discovery configuration.
#
# Discovery is the way the agents that should be managed are found.
# Replicante Core then interacts with agents by initiating connections out to them.
discovery:
  # Discovery backends configuration.
  #
  # Backends are a way to support an extensible set of discovery systems.
  # Each backend has its own options as described below.
  #
  # Available backends are:
  #
  #   * `files`: discover agents from local configuration files.
  backends:
    # The `files` backend discovers agents from files.
    #
    # It can be useful to delegate discovery to unsupported systems.
    # Examples are configuration management tools (ansible, chef, puppet, ...).
    #
    # This is a list of files that are periodically read to perform discovery.
    # When running replicated nodes for HA users must ensure every node has the same set of files.
    files: []

  # Interval (in seconds) to wait between agent discovery runs.
  interval: 60

  # Number of cycles before this node will re-run an election, 0 to disable re-runs.
  #
  # Having the system re-run elections continuously ensures that failover procedures are
  # exercised constantly and not just in case of errors.
  # You do not want to discover that failover does not work when a primary fails
  # and nothing picks up after it.
  #
  # With the default interval, this default will cause a re-election about every 3 hours.
  term: 10800


# The section below is for events related configuration.
events:
  # Configuration of periodic snapshots of tracked clusters/agents state.
  snapshots:
    # Enables/disables the emission of snapshot events.
    enabled: true

    # Frequency (expressed as number of cluster state fetches) of snapshot emission.
    #
    # Snapshots may be emitted more often than requested.
    # Precautions are taken to emit snapshots at least this frequently but there is no guarantee
    # this will happen, especially in the presence of some prolonged (total or partial) failures.
    frequency: 60

  # Configuration of the streaming platform events are emitted to.
  stream:
    # The backend to emit events to.
    # 
    # !!! DO NOT CHANGE AFTER INITIAL CONFIGURATION !!!
    # This option is to allow users to choose a supported software that best fits
    # their use and environment.
    #
    # Available options are:
    #
    #   * 'store': use the configured datastore as the backend for the events stream too.
    backend: 'store'

    # Any backend-specific option is set here.
    # The available options vary from backend to backend and are documented below.
    #
    # *** None available at this time ***
    #options:


# The section below is for logging configuration.
logging:
  # Flush logs asynchronously.
  # 
  # Pro:
  #     Async log flushing is more efficient as processes
  #     are not blocked waiting for logging backends to complete.
  # 
  # Con:
  #     If the process crashes logs in the buffer may be lost.
  #
  # Recommendation:
  #     Keep async logging enabled unless replicante is crashing
  #     and the logs don't have any indications of why.
  #
  #     Async logging may also be disabled in testing, debugging,
  #     or developing environments.
  async: true

  # Logging backend configuration.
  backend:
    # The backend to send logs to.
    # This option also determines the format and destination of logs.
    #
    # Available options:
    #
    #   * 'json': prints JSON formatted logs to standard output.
    #   * 'journald': sends logs to systemd journal (if enabled at compile time).
    name: json

    # Any backend-specific option is set here.
    # The available options vary from backend to backend and are documented below.
    #
    # *** None available at this time ***
    #options:

  # The minimum logging level.
  #
  # Available options:
  #
  #   * 'critical'
  #   * 'error'
  #   * 'warning'
  #   * 'info'
  #   * 'debug' (only available in debug builds)
  level: info

  # Advanced level configuration by module prefix.
  #
  # The keys in this map are used as prefix matches against log event modules.
  # If a match is found the mapped level is used for the event.
  # If no match is found the `level` value is used as the filter.
  #
  # Example:
  #
  #     modules:
  #       'hyper::server': debug
  #       'rdkafka': error
  #
  # To find out what modules are available you can set `level` to DEBUG
  # and enable `verbose` logging to see all logs.
  # Once you know what logs you are looking for you can undo the changes to `level` and `verbose`
  # and add the module prefix you need to the `modules` option.
  modules: {}

  # Enable verbose debug logs.
  #
  # When DEBUG level is enabled, things can get loud pretty easily.
  # To allow DEBUG level to be more useful, only application events are emitted at
  # DEBUG level while dependency events are emitted at INFO level.
  #
  # Verbose mode can be used in cases where DEBUG level should be enabled by default
  # on all events and not just the application logs.
  verbose: false


# The section below is for storage configuration.
storage:
  # The database to use for persistent storage.
  #
  # !!! DO NOT CHANGE AFTER INITIAL CONFIGURATION !!!
  # This option is to allow users to choose a supported database that best fits
  # their use and environment.
  #
  # Available options:
  #
  #   * 'mongodb' (recommended)
  backend: mongodb

  # Any backend-specific option is set here.
  # The available options vary from backend to backend and are documented below.
  #
  # MongoDB options:
  options:
    # Name of the MongoDB database to use for persistence.
    #
    # To change this option you will need to "Update by rebuild".
    # See the documentation for more details on this process.
    db: replicante  # (recommended)

    # URI of the MongoDB Replica Set or sharded cluster to connect to.
    uri: mongodb://localhost:27017/


# Task workers enabling configuration.
#
# For Replicante to function correctly ALL queues need to be worked on.
# By default, all queues are enabled on all nodes.
#
# While this allows the configuration to stay simple (all nodes in the cluster are identical),
# more advanced configuration options based around specialised node roles can be used to
# scale replicante deployments in a more efficient way (because roles can be scaled based on
# each role needs rather then scaling all roles to the needs of the biggest role).
#
# This is similar to the `components` configuration section but targeted at queues.
# Processes MUST have the `workers` component enabled on the process for tasks to be processed.
task_workers:
  # Default status for all task workers that are not explicitly enabled/disabled.
  _default: true

  # Enable cluster state refresh and aggregation task processing.
  cluster_refresh: null


# The section below is for the tasks system configuration.
tasks:
  # The queue system to use for async tasks.
  #
  # !!! DO NOT CHANGE AFTER INITIAL CONFIGURATION !!!
  # This option is to allow users to choose a supported software that best fits
  # their use and environment.
  #
  # Available options:
  #
  #   * 'kafka' (recommended)
  backend: 'kafka'

  # Any backend-specific option is set here.
  # The available options vary from backend to backend and are documented below.
  #
  # Kafka options:
  options:
    # Acknowledgement level for published messages.
    #
    # Possible values are:
    #
    #   * 'all': all in-sync-replicas for the partition published to must ack messages
    #            (default; required for consistency; see kafka documentation for details).
    #   * 'leader_only': only the leader of the partition published to must ack messages.
    #   * 'none': no acknowledgements are required for produced messages.
    ack_level: 'all'

    # Comma separated list of seed brokers.
    brokers: 'localhost:9092'

    # Number of attempts to commit offsets before giving up and recreating the client.
    commit_retries: 10

    # Worker session keepalive heartbeat interval (in milliseconds).
    heartbeat: 3000

    # Prefix to be placed in front of queue names to derive topic names.
    queue_prefix: 'task'

    # Kafka-specific timeout options.
    timeouts:
      # Timeout (in millisecond) for non-topic requests.
      metadata: 60000

      # Timeout (in milliseconds) for tasks to be acknowledged.
      request: 5000

      # Timeout (in milliseconds) after which workers are presumed dead by the brokers.
      session: 10000

      # Default timeout (in milliseconds) for network requests.
      socket: 60000

  # Number of task processing threads to spawn.
  #threads_count: 8 * number of CPUs


# Timeouts configured here are used throughout the system for various reasons.
timeouts:
  # Time (in seconds) after which API requests to agents are failed.
  agents_api: 15

      
# The section below is for distributed tracing configuration.
tracing:
  # The distributed tracing backend to integrate with.
  #
  # Available options:
  #
  #   * 'noop'
  #   * 'zipkin'
  backend: noop

  # Any backend-specific option is set here.
  # The available options vary from tracer to tracer and are documented below.
  #
  # Zipkin options
  #options:
  #  # (required) The service name for this zipkin endpoint.
  #  service_name: replicante
  #
  #  # (required) List of kafka seed hostnames.
  #  kafka:
  #    - HOST1:9092
  #    - HOST2:9092
  #
  #  # The kafka topic to publish spans to.
  #  topic: zipkin
```


### Timeouts
Replicante is an event-based, distributed, system.
As such we often expect things to happen within time limits but there is no guarantee that
they will in fact happen at all.

Timeouts are used to ensure that miss-functioning, slow, or unresponsive elements
(agents, processes, dependencies, ...) do not permanently or severely impact the entire system.

The timeout related options allow operators to tune the level of sensibility:

  * Low timeouts improve system responsiveness but require more reliable elements.
  * High timeouts are more forgiving of transient issues at the expense of responsiveness.


Available timeout options:

  * `timeouts.agents_api`: timeout applied to all agent HTTP requests.

Some of replicante dependencies (DB, coordinator, ...) also support timeouts that are
documented along the other options for each dependency.
