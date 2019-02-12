---
id: version-0.1.0-best-checklists
title: Best practices checklists
original_id: best-checklists
---

Support cluster and node level items that can be checked for presence/absence then build reports around this.

Things to think of:

  * Is latest version/is update available?
  * Is TLS required for connections?
  * Are there CVEs issued for the current version?

Start with focus on standard items that are applied to everything.
Quickly expand to custom items with partial agent support.


## Ideas

  * Add agent endpoint to return check results (are checks lightweight enough to
    assume they can be triggered by the endpoint itself?)
  * Endpoint returns a dictionary of `{<ID>: {"status": <bool>, "message": <string>}}`.
  * Core can poll the results:
    * Less frequently then for other state?
    * On demand to generate reports?
  * Data files used by core (imported in DB?) to attach meaning to items by ID:
    * Display text
    * Short description
    * Link to details
    * Is desired/undesired item?
    * Severity of presence/absence
