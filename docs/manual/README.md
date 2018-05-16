# Welcome ...
... to safe datastore automation.

Replicante is an open source data-driven automation system built specifically for datastores.

Automation is a known concept and there are some open source frameworks to implement it.
To remain general purpose, these frameworks mostly focus on providing an event bus and task
scheduling with APIs to emit events and rules to react to them.

While very powerful, this approach means it is up to the user to implement automations and tasks
as well as monitoring triggers.
The result often is overly-specific, fragile, set-ups that are hard to manage and share.

Replicante aims to avoid some of these shot-comings at the expense of generalisation:
a [well defined specification](https://www.replicante.io/docs/specs/master/) document determines
what a datastore is and can do.
Armed with this knowledge replicante can natively emit events with useful and consistent context
regardless of software in use or its version.


### Where to start?

  * Read the [architecture overview](first-steps/architecture.md) to understand how the system fits together.
  * Work through the [quick start](first-steps/quickstart.md) steps to setup a docker and docker-compose local playground.
  * Checkout the [features showcase](first-steps/features.md) if you are looking for something specific.
