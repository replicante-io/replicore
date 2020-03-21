use std::sync::Arc;

use bson::bson;
use bson::doc;
use bson::Bson;
use failure::Fail;
use failure::ResultExt;
use mongodb::coll::options::FindOptions;
use mongodb::db::ThreadedDatabase;
use mongodb::Client;
use mongodb::ThreadedClient;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;

use replicante_externals_mongodb::operations::find_with_options;
use replicante_models_core::events::Event;

use super::super::EventsInterface;
use super::constants::COLLECTION_EVENTS;
use super::constants::EVENTS_FILTER_NOT_SNAPSHOT;
use super::document::EventDocument;
use crate::store::events::EventsFilters;
use crate::store::events::EventsOptions;
use crate::Cursor;
use crate::ErrorKind;
use crate::Result;

/// Events operations implementation using MongoDB.
pub struct Events {
    client: Client,
    db: String,
    tracer: Option<Arc<Tracer>>,
}

impl Events {
    pub fn new<T>(client: Client, db: String, tracer: T) -> Events
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let tracer = tracer.into();
        Events { client, db, tracer }
    }
}

impl EventsInterface for Events {
    fn range(
        &self,
        filters: EventsFilters,
        opts: EventsOptions,
        span: Option<SpanContext>,
    ) -> Result<Cursor<Event>> {
        let mut options = FindOptions::new();
        options.limit = opts.limit;
        options.sort = Some(doc! {"$natural" => if opts.reverse { -1 } else { 1 }});

        let mut filter = Vec::new();
        if let Some(cluster_id) = filters.cluster_id {
            // Include events without a cluster ID to support cmobined system events.
            filter.push(Bson::from(doc! {"$or": [
                {"payload.cluster_id" => {"$eq" => cluster_id}},
                {"payload.cluster_id" => {"$exists" => false}},
            ]}));
        }
        if let Some(event) = filters.event {
            filter.push(Bson::from(doc! {"event" => {"$eq" => event}}));
        }
        if filters.exclude_snapshots {
            filter.push(Bson::from(doc! {
                "event" => EVENTS_FILTER_NOT_SNAPSHOT.clone()
            }));
        }
        if filters.exclude_system_events {
            filter.push(Bson::from(
                doc! {"payload.cluster_id" => {"$exists" => false}},
            ));
        }
        if let Some(start_from) = filters.start_from {
            filter.push(Bson::from(doc! {"timestamp" => {"$gte" => start_from}}));
        }
        if let Some(stop_at) = filters.stop_at {
            filter.push(Bson::from(doc! {"timestamp" => {"$lte" => stop_at}}));
        }
        let filter = if !filter.is_empty() {
            doc! {"$and" => filter}
        } else {
            doc! {}
        };
        let collection = self.client.db(&self.db).collection(COLLECTION_EVENTS);
        let cursor = find_with_options(
            collection,
            filter,
            options,
            span,
            self.tracer.as_deref(),
        )
        .with_context(|_| ErrorKind::MongoDBOperation)?
        .map(|item| item.map_err(|error| error.context(ErrorKind::MongoDBCursor).into()))
        .map(|result: Result<EventDocument>| result.map(Event::from));
        Ok(Cursor::new(cursor))
    }
}
