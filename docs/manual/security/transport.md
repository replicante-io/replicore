# Agent transports
Replicante core issues commands to the agents.
This design simplifies the agent interaction logic and allows for dynamic scaling controlled at core.

By default, communication with agents is performed over HTTP(s) as detailed below.
This approach makes the system easy to develop, debug, and interact with.
Using HTTP(s) as the transport layer also allows access to many existing tools for any need.

Replicante may at some point include other agent transports.


{% hint style="working" %}
**Unimplemented feature warning**

This page makes several mentions to **agent actions**.
This feature is not yet implemented but it will be a key part of the system.
For that reason they are planned for from the early stages.
{% endhint %}


## Transport security
Because a network is (likely) involved in the core-agent communication there are
some security aspects that must be considered and precautions to take.

There are two main ways for the transport to be abused:

  * Agent information and monitoring data could be faked.
    This would lead Replicante to infer the incorrect state of the node and issue corrective
    actions that could harm the healthy node.
  * Actions sent to agents could be faked.
    This would cause the agents to take actions that are issued with malignant intent.

There is also the possibility of packets being dropped by the transport layer.
This could result in a luck of visibility and/or an inability to issue actions to the node.
Because such event is indistinguishable from a regular network outage there is not much
that can be done to defend the system against this kind of attacks.

Replicante delegates securing the network to the transport layer.


## HTTP(s) transport
The HTTP transport is the easiest to use but also the least secure.

With this transport Replicante core act as an HTTP client for the agent.
Connections are established by replicante core and closed when no longer needed to avoid all
the complexity of long-running TCP connections (i.e, need for heart-beats, reconnect logic)
although this comes at the cost of repeated TPC connection handshakes.

As mentioned, HTTP is an insecure protocol but there are ways to add security to it.
Most notable is the HTTPS protocol which solves the agent's identity and message
integrity part of the equation.

Once the identity of the agent is verified by HTTPS and the channel is encrypted
it is possible to verify the identity and message integrity by signing messages:

  * Replicante core will use a signing (private) key to sign sensitive messages before they are sent.
  * Replicante agents will use a verification (public) key to verify the signature of messages.

The HTTP transport was mainly inspired by the advantages shown by
[Prometheus](https://prometheus.io/docs/introduction/faq/#why-do-you-pull-rather-than-push?)
but also for the added benefit of a simpler scalability architecture for replicante core.


### Configuration
At the time of writing HTTPS can be configured on agents through an HTTPS proxy server.