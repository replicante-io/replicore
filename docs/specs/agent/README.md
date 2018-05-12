# The Agent Protocol
> **[warning] Alpha state disclaimer**
>
> The protocol defined below is in early development cycle
> and is subject to (potentially breaking) change.

The Agents interface is a JSON encoded HTTP API.  
The API is versioned so that breaking changes can be rolled out gradually
with version compatibility "windows".


## Agent information
{% rest %}
  {% get "/api/v1/info/agent" %}
    Returns information about the agent itself

  {% get "/api/v1/info/datastore" %}
    Returns information about the datastore
{% endrest %}

[Details aboud these endpoints](./info.md)


## Shards information
{% rest %}
  {% get "/api/v1/shards" %}
    Returns detailed information about shards
{% endrest %}

[Details aboud these endpoints](./shards.md)
