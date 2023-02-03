<!-- markdownlint-disable MD022 MD024 MD032 -->
# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Introduce `platform.replicante.io/node.provision` action.
- Store `Namespace` objects in the primary store.
- Store `Platform` objects in the primary store.

### Changed
- **BREAKING**: Configuration `sentry_capture_api` is now an on/off boolean.
- **BREAKING**: Migrate `ClusterDiscovery` to `replisdk` provided model.
- **BREAKING**: Updated minimum rust version to 1.60.0.
- Capture only server-side errors in Sentry.
- Update dependencies.

## [0.7.2] - 2022-09-25
### Changed
- Emit `ACTION_CHANGED` events on action approve and disapprove.
- Emit `ACTION_CHANGED` events when node actions fail to schedule.
- Emit `ACTION_ORCHESTRATOR_CHANGED` events on orchestrator approve and disapprove.
- Errors during node sync are logged as info instead of debug.
- Node action approve and disapprove endpoints reject requests for actions not pending approval.
- Node action approve and disapprove endpoints reject requests for missing actions.
- Support debug log level in release builds.

### Fixed
- Fixed loading node and orchestrator actions from the database.

## [0.7.1] - 2022-09-14
### Changed
- **HOTFIX**: Use cargo resolver v2 as resolver v1 adds test code to releases and corrupts them.

## [0.7.0] - 2022-09-13
### Added
- Cluster discovery dynamically configured with `apply`.
- Discovery settings apply and delete events.
- Introduce `core.replicante.io/debug.*` actions: `counting`, `fail`, `ping` and `success`.
- Introduce `core.replicante.io/http` action.
- List and delete `DiscoverySettings` objects (API and `replictl`).
- Orchestrate reports for details on the latest attempt to sync a cluster.
- Orchestrator actions, executing during cluster orchestration.
- Reject any object with a non `default` namespace.
- Synthesise lag metrics for shards that don't report it.
- Synthetic in-memory Cluster Views.

### Changed
- **BREAKING**: The `events.stream` config block is no longer nested and is now simply `events`.
- **BREAKING**: Refactor event partition keys for streams.
- Populate view DB from the events stream.
- Refactor cluster discovery.
- Refactor cluster orchestration (also know as refresh).
- Start aliasing [agent] actions to node actions.

### Fixed
- Added Grafana API friendly text for missing events.
- Ensure `replictl` contexts store is flushed to disk before exiting.
- WebUI `/cluster/{cluster_id}/action/{action_id}` handles `GET` requests instead of `POST`.

### Removed
- **BREAKING**: Discovery backends defined in the config file are ignored.
- **BREAKING**: Dropped snapshot events.
- The `events_indexer` component was replaced by the `viewupdater` component.

## [0.6.0] - 2020-05-28
### Added
- Support for TLS and mTLS.

### Changed
- **BREAKING**: Remove MongoDB index validation (alpha driver does not support fetching indexes).
- **BREAKING**: Rework `replictl sso` into `replictl context`.
- Replaced deprecated iron with actix-web.
- Update all dependencies to latest available versions.

## [0.5.0] - 2020-03-07
### Added
- `replictl apply` command to request actions/changes.
- `replictl cluster refresh` command to trigger a cluster refresh task.
- API to `apply` change requests.
- API to fetch action details.
- API to search cluster actions.
- Actions and Actions API added to the specification.
- Actions and actions history stored in the view store.
- Agent actions sync and events.
- Centralised actions scheduling.
- Support action approval before scheduling.
- Support mutually authenticated HTTPS transport.

### Changed
- **BREAKING**: Moved existing `replictl` commands to new `repliadmin` CLI tool.
- **BREAKING**: Rebuilt `replictl` CLI tool.
- **BREAKING**: Refactor `Event`s model.

## [0.4.0] - 2019-07-15
### Added
- Cluster specific agents API (`/cluster/:id/agents`).
- Cluster specific events API (`/cluster/:id/events`).
- Cluster specific nodes API (`/cluster/:id/nodes`).
- HTTP Discovery.
- Partial release automation.

### Changed
- **BREAKING**: Change `storage` configuration to `storage.primary`.
- **BREAKING**: Store every event in the "view" database.

### Removed
- **BREAKING**: File Discovery.

## [0.3.1] - 2019-07-04
### Added
- Common streams logic.
- Emit events to stream (and not store).
- Follow events stream.
- Kafka as a stream backend.
- Relay events from stream to store.

## [0.3.0] - 2019-06-16
### Added
- A better thread story.
- Additional API server configuration options.
- Cluster refresh operation tracing.
- Cluster refresh request endpoint.
- Graceful shutdown.
- Introduce an `/api/unstable` API "version".
- Optional `display_name` discovery attribute.
- Sentry integration.
- Threads introspection API.
- Update checker at start (disabled by default).
- `/api/unstable/introspect/health` endpoint.
- `/api/unstable/introspect/self` endpoint.

### Changed
- **BREAKING**: Agent client uses `unstable` API.
- **BREAKING**: Cluster ID and Friendly name.
- **BREAKING**: Rename incorrectly named v1 API as unstable.
- **BREAKING**: Rename node's `name` to `node_id`.
- **BREAKING**: Rename shard's `id` to `shard_id`.
- Improve data consistency in cases where data is unavailable.
- Move cluster metadata to aggregation pipeline.
- Reworked primary store interface.
- Standardise metrics names.

### Removed
- **BREAKING**: Removed nonsensical ordering on some models.

## [0.2.1] - 2019-03-28
### Added
- Add `rustfmt` to CI and move in that direction.
- Add coordinator version to `replictl versions`.
- Add queue system version to `replictl versions`.
- Contributing Guidelines.
- DCO requirement.
- Docker images.
- GitHub issue template.
- Introduce Code of Conduct.
- Pre-built binaries.
- Travis CI builds.

### Changed
- Improve docker-based dev environment and auto-initialise it.
- Improve playgrounds usability.
- Replace `error-chain` with `failure`.
- Standardise logging across core and agents.

## [0.2.0] - 2019-02-20
### Added
- Add scan method to events stream interface.
- Async tasks and workers framework.
- Configurable Agent API calls timeout.
- Configurable components enabled in each process.
- Configurable kafka tasks ack level.
- Configurable task queues enabled in each process.
- Distributed coordinator system.
- Emit log messages from the `log` crate.
- Emit new `AGENT_INFO_CHANGED` event.
- Emit new `AGENT_INFO_NEW` event.
- Emit new `CLUSTER_CHANGED` event.
- Emit new `NODE_CHANGED` event.
- Emit new `NODE_NEW` event.
- Emit new `SHARD_ALLOCATION_CHANGED` event.
- Emit new `SHARD_ALLOCATION_NEW` event.
- Events streaming interface (backed by store).
- Fine-grained log level configuration.
- Grafana annotations backend (through JSON data source).
- HA `discovery` component.
- Periodically emit snapshot events.
- ROADMAP.md to document "nearby" versions and their "aim".

### Changed
- **BREAKING**: Flatten encoded events structure (`data`, `event`, `timeout` are all root attributes).
- **BREAKING**: Rename `*Recover` events.
- **BREAKING**: Rename `DatastoreDown` to `NodeDown`.
- **BREAKING**: Rename `DatastoreRecovered` to `NodeUp`.
- **BREAKING**: Rename `EventData` to `EventPayload`.
- **BREAKING**: Replace `AgentStillDown` with `AgentDown`.
- **BREAKING**: Replace `DatastoreStillDown` with `NodeDown`.
- **BREAKING**: Replace `replictl` progress bar with periodic logs.
- **BREAKING**: Replace store `recent_events` with `events(filters, options)` and return an iterator.
- **BREAKING**: Rework `AGENT_NEW` to include only cluster and host.
- **BREAKING**: Update models.
- Emit agent status change after emitting `AGENT_NEW` events.
- Move logging code to common crate.
- Name background threads.
- Refactor metrics initialisation code.
- Start move to the `failure` crate.

### Fixed
- Better handle HTTP errors returned by agents.
- Fix persistence and querying of discoveries.

## 0.1.0 - 2018-06-28
### Added
- API interface.
- Agent client crate.
- Command line control tool (replictl).
- Docker compose file for development.
- Emit "new cluster" events.
- Emit "new agent" events.
- Emit agent status change events.
- Fetch agent state.
- File agent discovery.
- Storage interface.
- Store clusters and nodes information.
- User manual (documentation).
- Validate contents of the store.
- WebUI endpoints for initial UI.

[Unreleased]: https://github.com/replicante-io/replicante/compare/v0.7.2...HEAD
[0.7.2]: https://github.com/replicante-io/replicante/compare/v0.7.1...v0.7.2
[0.7.1]: https://github.com/replicante-io/replicante/compare/v0.7.0...v0.7.1
[0.7.0]: https://github.com/replicante-io/replicante/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/replicante-io/replicante/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/replicante-io/replicante/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/replicante-io/replicante/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/replicante-io/replicante/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/replicante-io/replicante/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/replicante-io/replicante/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/replicante-io/replicante/compare/v0.1.0...v0.2.0
