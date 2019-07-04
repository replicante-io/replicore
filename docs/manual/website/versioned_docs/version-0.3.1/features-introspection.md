---
id: version-0.3.1-features-introspection
title: Introspection
sidebar_label: Introspection
original_id: features-introspection
---

In an ideal world software, once installed and configured, runs perfectly without ever needing attention.
Of course reality tells a different story.

Software evolves, software has bugs, transient network issues, miss-configurations ...  
Many things can go wrong and the symptoms are often unclear.
Distributed systems also mean that errors often propagate across processes and servers
so the location an error is reported is far from the location where the error originated.

On top of all that, distributed systems are complex to follow and
simple questions about correct functioning become hard to answer.

Replicante is a distributed system and as such it is subject to all the above complications.
To help users and administrators understand and manage installations, as well as troubleshoot issues,
Replicante provides a set of features to introspect the system and trace its activity.


## Events trail
Replicante is an event-driven system at its core.
Because of that, most activities of the system can be explained and monitored
by looking at the events stream.

The [events](features-events.md) section explains how to view and programmatically follow events.


## Metrics
Information about internal operation of replicante is exposed through metrics.
These can be used to monitor the health and activity of a process as well as its performance.

Metrics are exposed in [Prometheus](https://prometheus.io/)
format by the API endpoint `/api/unstable/introspect/metrics`.


## Logging
Logging is a good way to see exactly what one system was doing at a precise point in time.
Replicante provides structured logging so administrators can see what is happening and in what context.

By itself this is needed but not that great.
The real power of structured logging comes in with centralised log collection:
the logs from every server are collected and indexed in a central location along with other services.


### Configuration
Various logging backends are supported so that replicante can fit into your infrastructure
and some options are provided to user regardless of the backend of choice.
All options are under the `logging` section.
The details are documented in the [configuration reference](admin-config.md).

Below are the supported backends:

  * `json` (default) outputs logs to standard output in JSON format:
    * The output is not the easiest to read directly.
    * It works well with process supervisors that expect logs from standard output (i.e, docker).
    * Lines can be processed by any tools that understand JSON (`fluentd`, `logstash`, `jq` or crafted scripts).
  * `journald` sends logs to journald directly (systemd's logging facility):
    * `journald` is available only if enabled at compile time.
    * `journald` is requires a server running systemd.


## Distributed Tracing
Following the details of an operation from start to finish when it spans several servers
can be a challenge.
Thankfully there are tools to address this challenge: distributed tracers.

Distributed tracers are central systems that collect segments of operations from different servers
and combine them together to show the entire story of a full operation.


### Configuration
Replicante supports integration with some distributed tracing tools compatible with the
[OpenTracing](http://opentracing.io/) specification.

By default distributed tracing is disabled but it can be [configured](admin-config.md)
with the options under the `tracing` section.


## Sentry
[Sentry](https://sentry.io/) is a really powerful, open source, tool to collect
and understand errors reported by applications.

Replicante integrates with sentry to inform operators about unexpected situations.

Not everything reported is an error, and not every error is critical.
Some errors may not even require attention but are instead an indication of temporary
issues: transient network issues and dependencies failover may lead to errors
being reported to sentry.
These are symptoms of an external conditions that you may or may not need to look into.


## Introspection API
If what you need is not available among the tools above, a more machine-oriented
[introscpection API](api-introspection.md) is available.

Keep in mind that this API is meant for advanced operators and developers and may be
useless, even missleading, without more context and a deeper knowledge of the code.
