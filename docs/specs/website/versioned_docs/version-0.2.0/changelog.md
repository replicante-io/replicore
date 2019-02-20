---
id: version-0.2.0-changelog
title: Model Change Log
sidebar_label: Model Change Log
original_id: changelog
---

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## Unreleased
### Changed
- **BREAKING**: Encode shard roles in lowercase.
- **BREAKING**: Replaced shard's `last_op` with a `commit_offset`.
- **BREAKING**: Replication lag has a specified unit (no longer assumed to be seconds).

### Removed
- Half-thought performance stats.

## 0.1.0 - 2018-01-28
### Added
- Initial model definition.
