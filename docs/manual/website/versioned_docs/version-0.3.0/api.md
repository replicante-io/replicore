---
id: version-0.3.0-api
title: API Reference
sidebar_label: API Reference
original_id: api
---

Replicante provides access to the system through an HTTP JSON API.
This section provides details of this public interface.

<blockquote class="warning">

**The public API is still being designed as the system reaches maturity**

Access to features through the public API is limited while features stabilise.

</blockquote>


## Versioning
The API is versioned, with a single major version (i.e, `v1`, `v2`, ...),
so that breaking changes will have limited impacts on users.

Additional endpoints, additional response attributes, and bug fixes that do not remove
attributes or change types are considered minor changes (the version is not changed).

Changes to attribute types, removal of response attributes, and removal of endpoints
are considered breaking changes and the version is incremented.

### Unstable
API design is difficult and schemas are hard to get right the first time around.
Incorrect design leads to poor performance, difficult use, and often additional complexity
to maintain a sub-optimal solution.
On the other hand continuous changes to an API makes it difficutl to build an
ecosystem of tools and integrations.

In the hope of achieving a sensible balance, endpoints start their life under the
`/api/unstable` "version" where they can change fast and break often.
In [semver](https://semver.org/) terms all endpoints under `/api/unstable` are not part of
the public API.
Once enough iterations have passed and endpoints are stable for the foreseeable future they
are promoted to the current stable version and become subject to breaking changes restrictions.

<blockquote class="info">

While the unstable API is subject to change, we do want to encourage early adopters
to use such endpoints when possible and report issues (performance, usability, anything really!).

Only this way we can iterate over the API and improve it before it becomes stable.

</blockquote>


## API Configuration
By default, all supported API versions and unstable endpoints are made available
when the system starts.
For advanced users and setups, the configuration file also allows control over which
endpoints are made available.

Endpoints that are provided by specific components, like the
[grafana integration](features-events.md#grafana-annotations),
are only enabled it their component is as well.

On top of those options, additional "filters" can be applied to pick which
sets of endpoints should be made available by replicante processes.
These are configured as boolean options under the `api.trees` configuration section.
Available options are:

  * `introspect`: disable all (stable and unstable) introspection API endpoints.
  * `unstable`: disable all unstable API endpoints.
