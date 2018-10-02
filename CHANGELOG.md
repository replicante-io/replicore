# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Configurable Agent API calls timeout.
- Emit new `AGENT_INFO_NEW` event.
- Emit new `CLUSTER_CHANGED` event.
- Emit new `NODE_NEW` event.
- Emit new `SHARD_ALLOCATION_NEW` event.

### Changed
- **BREAKING**: Flatten encoded events structure (`data`, `event`, `timeout` are all root attributes).
- **BREAKING**: Rename `*Recover` events.
- **BREAKING**: Rename `DatastoreDown` to `NodeDown`.
- **BREAKING**: Rename `DatastoreRecovered` to `NodeUp`.
- **BREAKING**: Rename `EventData` to `EventPayload`.
- **BREAKING**: Replace `AgentStillDown` with `AgentDown`.
- **BREAKING**: Replace `DatastoreStillDown` with `NodeDown`.
- **BREAKING**: Rework `AGENT_NEW` to include only cluster and host.
- **BREAKING**: Update models.
- Emit agent status change after emitting `AGENT_NEW` events.
- Move logging code to common crate.

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


[Unreleased]: https://github.com/replicante-io/replicante/compare/v0.1.0...HEAD
