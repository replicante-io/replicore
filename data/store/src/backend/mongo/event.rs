use bson;
use bson::Bson;
use bson::UtcDateTime;

use mongodb::Client;
use mongodb::ThreadedClient;
use mongodb::coll::Collection;
use mongodb::coll::options::FindOptions;
use mongodb::db::ThreadedDatabase;

use replicante_data_models::Event;
use replicante_data_models::EventPayload;

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
        let event: EventWrapper = event.into();
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
        options.limit = Some(i64::from(limit));
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
            let event = bson::from_bson::<EventWrapper>(bson::Bson::Document(doc))
                .chain_err(|| FAIL_RECENT_EVENTS)?;
            events.push(event.into());
        }
        Ok(events)
    }

    /// Returns the `events` collection.
    fn collection_events(&self) -> Collection {
        self.client.db(&self.db).collection(COLLECTION_EVENTS)
    }
}



/// A wrapper for the `Event` model to allow BSON to encode/decode timestamps correctly.
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct EventWrapper {
    #[serde(flatten)]
    pub payload: EventPayload,
    pub timestamp: UtcDateTime,
}

impl From<Event> for EventWrapper {
    fn from(event: Event) -> EventWrapper {
        EventWrapper {
            payload: event.payload,
            timestamp: UtcDateTime(event.timestamp),
        }
    }
}

impl From<EventWrapper> for Event {
    fn from(event: EventWrapper) -> Event {
        Event {
            payload: event.payload,
            timestamp: event.timestamp.0,
        }
    }
}
