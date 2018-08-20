# Agent information
{% hint style="working" %}
**Alpha state disclaimer**

The protocol defined below is in early development cycle
and is subject to (potentially breaking) change.
{% endhint %}


{% rest %}
  {% get "/api/v1/info/agent" %}
    Returns information about the agent itself
{% endrest %}
{% method %}
The following agent information MUST be included:

  * `version` information:
    * Version `number`: the [SemVer](https://semver.org/) agent version.
    * Version control `checkout`: VCS ID of the agent code that is running.
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
The following datastore information MUST be included:

  * `cluster`: datastore determined cluster name.
  * `kind`: datastore software managed by the agent.
  * `name`: datastore determined, cluster unique, node name.
  * `version`: the [SemVer](https://semver.org/) datastore version.

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
