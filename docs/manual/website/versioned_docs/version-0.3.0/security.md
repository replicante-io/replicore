---
id: version-0.3.0-security
title: Security Considerations
sidebar_label: Security Considerations
original_id: security
---

This section covers Replicante security design and features.

Replicante aims to provide a reasonable level of security by default.
Tuning will be required to ensure users make the best of the offered features.


## Design expectations
Replicante is developed with some expectation about data, users, and runtime environment:

  * Agents can be trusted and respect the [specification](https://www.replicante.io/docs/specs/).
  * Users that have access to the system can be trusted.
  * Replicante processes can trust each other.
  * Agent actions can be harmful so the authenticity and integrity of action requests MUST be guaranteed.
  * Collected data and generated events are sensitive information (precautions are taken to avoid
    unauthorised reads) but are not confidential (no permanent or irreparable harm is done if data
    is leaked).
  * Security of third party software (store, message bus, ...) is out of scope.


## Features
Replicante provides the following security-related features:

  * [Agents transport](security-transport.md) encryption and action signing.
  * [Events](features-events.md#stream-subscription) auditing.