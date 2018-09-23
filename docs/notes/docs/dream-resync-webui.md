---
id: resync-webui
title: Resyncable WebUI application
---

Build a completely independent WebUI application (with its own database and everything) 
that keeps itself in sync with Replicante core by replaying the events stream.

Design a solution that can fully re-sync itself just by looking at the available stream of events.
This would allow core to focus on its tasks, with data formats convenient for them,
and the UI to focus on efficient and effective user interaction and analytics.

The ability to fully resync the WebUI based on backend events means that any change in the UI
datastore or in the logic of how events are processed can be handled by simply drop the data
and re-consume the stream of events.

## Notes

  * The event stream emits complete snapshots of its internal data periodically
  * The UI can use the oldest snapshots to base its data on
  * The UI can incrementally derive analytics and other features based on this data
  * Newer snapshots can be used to validate state in the UI DB and warn of inconsistencies
  * Support resync of individual clusters/organisations
  * Figure out how to deal with non-monitored data (teams/users):
    * Always use core/shared system for authentication and authorisation
    * Lazily fetch additional data from core when encountered during processing or user requests?
