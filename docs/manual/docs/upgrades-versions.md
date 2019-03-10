---
id: upgrades-versions
title: Version Compatibility
sidebar_label: Version Compatibility
---

Replicante follows the [semantic versioning](https://semver.org/) specification to
indicate changes that can pose compatibility issues among versions.

The "public API" of replicante core is comprised of a set of different interfaces:

  * The public endpoints of the API component.
  * The data schema of elements stored in external systems (i.e, storage, coordinator, message bus).
  * The supported agent communication protocols.


The table below shows a summary of supported agent protocols and minimum upgrade version:

| Replicante Version | Supported Agent API | Upgrade from |
| ------------------ | ------------------- | ------------ |
| 0.2.x              | v1                  | N/A*         |
| 0.1.x              | v1                  | -            |

*Versions below 1.0.0 are early development released with large breaking changes and
unlikely realistic need for rolling update support.