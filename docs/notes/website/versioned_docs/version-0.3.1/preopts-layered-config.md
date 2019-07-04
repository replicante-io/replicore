---
id: version-0.3.1-layered-config
title: Layered Configuration
original_id: layered-config
---

The current plan is to use global only configurations.
This keeps things simple and manageable.

As Replicante matures and introduces features like organisations,
a global configuration may become too limiting.

The idea is to create configuration layers, from more to less specific, that fall back
to less specific layers if the current layer does not have the requested configuration.


## Layers (most generic to most specific)

  1. Global configuration (effectively acting as defaults).
  2. Per-organisation configuration (when organisations are introduced).
  3. Per-cluster configuration (when orgs exist, this would presumably be rare).


## Why wait?

  * Needs are yet unkown.
  * Organisations will have a large impact on the design of this.
