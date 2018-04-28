use bson;
use bson::Bson;

use mongodb::Client;
use mongodb::ThreadedClient;
use mongodb::coll::Collection;
use mongodb::coll::options::FindOptions;
use mongodb::db::ThreadedDatabase;

use replicante_data_models::Event;

use super::super::super::Result;
use super::super::super::ResultExt;

use super::constants::FAIL_PERSIST_EVENT;
use super::constants::FAIL_RECENT_EVENTS;
use super::constants::COLLECTION_EVENTS;

use super::metrics::MONGODB_OPS_COUNT;
use super::metrics::MONGODB_OPS_DURATION;
use super::metrics::MONGODB_OP_ERRORS_COUNT;


/// Subset of the `Store` trait that deals with events.
pub struct EventStore {
    client: Client,
    db: String,
}

impl EventStore {
    pub fn new(client: Client, db: String) -> EventStore {
        EventStore { client, db }
    }

    pub fn persist_event(&self, event: Event) -> Result<()> {
        let document = bson::to_bson(&event).chain_err(|| FAIL_PERSIST_EVENT)?;
        let document = match document {
            Bson::Document(document) => document,
            _ => panic!("Event failed to encode as BSON document")
        };
        let collection = self.collection_events();
        MONGODB_OPS_COUNT.with_label_values(&["insertOne"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["insertOne"]).start_timer();
        collection.insert_one(document, None)
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["insertOne"]).inc();
                error
            })
            .chain_err(|| FAIL_PERSIST_EVENT)?;
        Ok(())
    }

    pub fn recent_events(&self, limit: u32) -> Result<Vec<Event>> {
        let mut options = FindOptions::new();
        options.limit = Some(limit as i64);
        options.sort = Some(doc!{"$natural" => -1});
        let collection = self.collection_events();
        MONGODB_OPS_COUNT.with_label_values(&["find"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["find"]).start_timer();
        let cursor = collection.find(None, Some(options))
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["find"]).inc();
                error
            })
            .chain_err(|| FAIL_RECENT_EVENTS)?;

        let mut events = Vec::new();
        for doc in cursor {
            let doc = doc.chain_err(|| FAIL_RECENT_EVENTS)?;
            let event = bson::from_bson::<Event>(bson::Bson::Document(doc))
                .chain_err(|| FAIL_RECENT_EVENTS)?;
            events.push(event);
        }
        Ok(events)
    }

    /// Returns the `events` collection.
    fn collection_events(&self) -> Collection {
        self.client.db(&self.db).collection(COLLECTION_EVENTS)
    }
}
