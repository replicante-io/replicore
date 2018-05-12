# Agent information
> **[warning] Alpha state disclaimer**
>
> The protocol defined below is in early development cycle
> and is subject to (potentially breaking) change.


{% rest %}
  {% get "/api/v1/info/agent" %}
    Returns information about the agent itself
{% endrest %}
{% method %}
The following information MUST be included:

  * `agent` information:
    * `version` information:
    * Version `number`: the [SemVer](https://semver.org/) agent version.
    * Version control `checkout`: VCS ID of the code that was build.
    * Version control `taint` status: indicates uncommitted changes to the code.

{% common %}
Example:
```json
{
  "version": {
    "number": "0.1.0",
    "checkout": "11a24d9c3940f60e527c571680d64e80e0889abe",
    "taint": "not tainted"
  }
}
```
{% endmethod %}


{% rest %}
  {% get "/api/v1/info/datastore" %}
    Returns information about the datastore
{% endrest %}
{% method %}
The following information MUST be included:

  * `datastore` information:
    * `cluster`: datastore determined cluster name.
    * `kind`: datastore software managed by the agent.
    * `name`: datastore determined, cluster unique, node name.
    * `version`: the datastore version.

{% common %}
Example:
```json
{
  "cluster": "replica-set-name",
  "kind": "MongoDB",
  "name": "host.domain.com:27017",
  "version": "3.4.5"
}
```
{% endmethod %}
