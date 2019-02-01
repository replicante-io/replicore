---
id: api
title: API Reference
sidebar_label: API Reference
---

Replicante provides access to the system through and HTTP JSON API.
This section provides details of this public interface.

<blockquote class="warning">

**The public API is still being designed as the system reaches maturity**

Access to features through the public API is limited while the feature stabilise.

</blockquote>


## Versioning
The API is versioned, with a single major version (i.e, `v1`, `v2`, ...),
so that breaking changes will have limited impacts on users.

Additional endpoints, additional response attributes, and bug fixes that do not remove
attributes or change types are considered minor changes (the version is not changed).

Changes to attribute types, removal of response attributes, and removal of endpoints
are considered breaking changes and the version is incremented.
