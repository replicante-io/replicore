---
id: core-bootstrap
title: Replicante bootstrapping procedure
---

Allow one-command start of a Replicante instance that can be used to configure
all datastores and dependences for Replicante itself, then generate a configuration
file to be used for production instances.

May be just a guide but will likely require code changes/extra features.
Most likely, in-memory version of the dependencies should be made available so the process can
start with no additional complexity or impairment.