## 0.3.0 - Introspection
- Standardise metrics names.
- Investigate yield and harvast.
- Check for available updates.
- Status endpoints.
- Cluster refresh request endpoint.


## 0.3.1 - Events stream
- Emit events to stream (and not store).
- Follow/consume streams by group.
- Relay events from stream to store.


## 0.4.0 - Split "view" database
- Move the indexed event collection to dedicated interface (still in mongo but different DB).
- Emit messages to kafka instead of the store.
- Store every event in the "view" database.


## 0.4.1 - Additional data in the UI
- Add agents information to the UI.
- Add shards information to the UI.
- Add cluster events to cluster view.
- Filter for cluster/system/all events.
- Expandable event payload box with easier to read JSON.


## 0.4.2 - More agents?
- Redis?
- Cassandra?

## 0.?.? - Actions?

## 0.?.? - Organisations/teams?

## 0.?.? - Rate limits?
