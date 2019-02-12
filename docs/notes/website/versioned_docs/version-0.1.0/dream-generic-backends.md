---
id: version-0.1.0-generic-backends
title: Generic backends interface
original_id: generic-backends
---

Replicante use of abstractions over dependencies is something I like as it allows faster
integrations with new tools/allows different users and use cases for different people.
Integrating with all possible options (things like all document stores, all coordinators, etc ...)
is not wise and likely not possible.

To allow use of tools where full integration is undesired/impossible,
create a GRPC (or other framework) client implementation and server specification.
Integrations with new tools can then be offloaded by implementing a GRPC/other server fulfilling the specification.


## Use cases:

  * Prototype new integrations
  * Integrating with ecosystems that do not work well with rust
  * Private/proprietary integrations that may not be sharded at large
