## 0.2.1 - Project improvements
- Add coordinator and task queue versions to `replictl versions`.
- Add milestones list to roadmap page.
- Automate docker-based dev setup.
- Standardise logging across core and agents.
- Pre-built binaries?
- DCO requirement?
- Dual license?


## 0.2.2 - Introspection
- Improve generated cluster data.
- Improve metrics names.
- Sentry integration.
- Trace agent discovery.
- Trace state fetching.


## 0.2.3 - Kafka events stream
- Emit events to stream (and not store).
- Follow/consume streams by group.
- Relay events from stream to store.


## 0.3.0 - Split "view" database
- Move the indexed event collection to dedicated interface (still in mongo but different DB).
- Emit messages to kafka instead of the store.
- Generate tasks out of events.
- Store every event in the "view" database.


## 0.3.1 - Additional data in the UI
- Add agents information to the UI.
- Add shards information to the UI.
- Add cluster events to cluster view.
- Filter for cluster/system/all events.
- Expandable event payload box with easier to read JSON.


## 0.3.2 - More agents?
- Redis?
- Cassandra?

## 0.?.? - Actions?

## 0.?.? - Organisations/teams?

## 0.?.? - Rate limits?
