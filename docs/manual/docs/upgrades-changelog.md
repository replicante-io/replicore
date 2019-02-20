---
id: upgrades-changelog
title: Change Log
sidebar_label: Change Log
---

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2019-02-XX
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
- Grafana annotations backend (through JSON datasource).
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


[0.2.0]: https://github.com/replicante-io/replicante/compare/v0.1.0...v0.2.0
