---
id: scaling-coordinator
title: Coordinator
sidebar_label: Coordinator
---

Replicante Core uses the coordinator to support its operations.
All operations performed by the coordinator are simple and small operations.

As such the expectation is for the coordinator not to need to be scaled horizontally but rather
vertically as the system grows (a properly sized coordinator cluster should be able to cope).
