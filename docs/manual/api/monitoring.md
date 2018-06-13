# Monitoring
The endpoints in this section provides information about the current state of the system.


{% rest %}
  {% get "/api/v1/metrics" %}
{% endrest %}
{% method %}
Prometheus-style metrics exposition.

{% common %}
```text
# HELP replicante_discovery_fetch_errors Number of errors during agent discovery
# TYPE replicante_discovery_fetch_errors counter
replicante_discovery_fetch_errors 2
# HELP replicante_discovery_loops Number of discovery runs started
# TYPE replicante_discovery_loops counter
replicante_discovery_loops 1
```
{% endmethod %}


{% rest %}
  {% get "/api/v1/version" %}
{% endrest %}
{% method %}
Return version information about the running code:

  * `commit`: the git commit the code was build from.
  * `taint`: indicates if the code was changed compared to the commit.
  * `version`: friendly semantic versioning string.

{% common %}
```json
{
	"commit": "bbe5ddf4b62608974a35335014b854a650e72f7c",
	"taint": "working directory tainted",
	"version": "0.1.0"
}
```
{% endmethod %}
