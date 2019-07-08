use std::ops::Deref;
use std::sync::Arc;

use bson::Bson;
use failure::ResultExt;
use mongodb::db::ThreadedDatabase;
use mongodb::Client;
use mongodb::ThreadedClient;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;

use replicante_externals_mongodb::operations::insert_one;
use replicante_models_core::Event;

use super::super::PersistInterface;
use super::constants::COLLECTION_EVENTS;
use super::document::EventDocument;
use crate::Error;
use crate::ErrorKind;
use crate::Result;

/// Persistence operations implementation using MongoDB.
pub struct Persist {
    client: Client,
    db: String,
    tracer: Option<Arc<Tracer>>,
}

impl Persist {
    pub fn new<T>(client: Client, db: String, tracer: T) -> Persist
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let tracer = tracer.into();
        Persist { client, db, tracer }
    }
}

impl PersistInterface for Persist {
    fn event(&self, event: Event, span: Option<SpanContext>) -> Result<()> {
        let collection = self.client.db(&self.db).collection(COLLECTION_EVENTS);
        let event = EventDocument::from(event);
        let document = bson::to_bson(&event).with_context(|_| ErrorKind::MongoDBBsonEncode)?;
        let document = match document {
            Bson::Document(document) => document,
            _ => panic!("Event failed to encode as BSON document"),
        };
        insert_one(
            collection,
            document,
            span,
            self.tracer.as_ref().map(|tracer| tracer.deref()),
        )
        .with_context(|_| ErrorKind::MongoDBOperation)
        .map_err(Error::from)
    }
}
