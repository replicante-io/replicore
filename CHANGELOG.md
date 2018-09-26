# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Configurable Agent API calls timeout

### Changed
- **BREAKING**: Update models.
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
- WebUI enpoints for initial UI.


[Unreleased]: https://github.com/replicante-io/replicante/compare/v0.1.0...HEAD
