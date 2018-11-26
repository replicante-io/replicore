## 0.2.0 - High Availability (Work In Progress)
- Introduce distibuted coordinator.
- Move agent fetch and aggregation to tasks.
- Make discovery worker HA (by using elections).


## 0.2.1 - Small Improvements
- Improve generated cluster data.


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


## 0.3.2 - Introspection
- Improve metrics names.
- Sentry integration.
- Trace agent discovery.
- Trace state fetching.


## ??? - Actions?
