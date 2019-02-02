---
id: version-0.1.0-upgrades-versions
title: Version Compatibility
sidebar_label: Version Compatibility
original_id: upgrades-versions
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
| 0.1.*              | v1                  | -            |
