---
id: scaling-coordinator
title: Coordinator
sidebar_label: Coordinator
---

Replicante Core uses the coordinator to support its operations.
All operations performed against the coordinator are simple and small.

As a result the coordinator is not expected to need to scale horizontally.
Rather vertical scaling as the system grows should be considered
(a properly sized coordinator cluster should be able to cope).
