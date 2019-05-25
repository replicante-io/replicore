use bson::Bson;
use failure::ResultExt;
use mongodb::coll::options::FindOptions;
use mongodb::db::ThreadedDatabase;
use mongodb::Client;
use mongodb::ThreadedClient;
use regex;

use replicante_data_models::ClusterMeta;
use replicante_data_models::Event;

use super::super::super::store::legacy::EventsFilters;
use super::super::super::store::legacy::EventsOptions;
use super::super::super::Cursor;
use super::super::super::ErrorKind;
use super::super::super::Result;
use super::super::LegacyInterface;
use super::common::find_one;
use super::common::find_with_options;
use super::common::insert_one;
use super::common::replace_one;
use super::constants::COLLECTION_CLUSTER_META;
use super::constants::COLLECTION_EVENTS;
use super::constants::EVENTS_FILTER_NOT_SNAPSHOT;
use super::constants::TOP_CLUSTERS_LIMIT;
use super::document::EventDocument;

/// Legacy operations implementation using MongoDB.
pub struct Legacy {
    client: Client,
    db: String,
}

impl Legacy {
    pub fn new(client: Client, db: String) -> Legacy {
        Legacy { client, db }
    }
}

impl LegacyInterface for Legacy {
    fn cluster_meta(&self, cluster_id: String) -> Result<Option<ClusterMeta>> {
        let filter = doc! {"cluster_id" => &cluster_id};
        let collection = self.client.db(&self.db).collection(COLLECTION_CLUSTER_META);
        find_one(collection, filter, None, None)
    }

    fn events(&self, filters: EventsFilters, opts: EventsOptions) -> Result<Cursor<Event>> {
        let mut options = FindOptions::new();
        options.limit = opts.limit;
        options.sort = Some(doc! {"$natural" => if opts.reverse { -1 } else { 1 }});

        let mut filter = Vec::new();
        if let Some(cluster_id) = filters.cluster_id {
            // Include events without a cluster ID to support cmobined system events.
            filter.push(Bson::from(doc! {"$or": [
                {"data.cluster_id" => {"$eq" => cluster_id}},
                {"data.cluster_id" => {"$exists" => false}},
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
            filter.push(Bson::from(doc! {"data.cluster_id" => {"$exists" => false}}));
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
        let cursor = find_with_options(collection, filter, options, None)?
            .map(|result: Result<EventDocument>| result.map(Event::from));
        Ok(Cursor(Box::new(cursor)))
    }

    fn find_clusters(&self, search: String, limit: u8) -> Result<Cursor<ClusterMeta>> {
        let search = regex::escape(&search);
        let filter = doc! { "$or" => [
            {"cluster_display_name" => {"$regex" => &search, "$options" => "i"}},
            {"cluster_id" => {"$regex" => &search, "$options" => "i"}},
        ]};
        let collection = self.client.db(&self.db).collection(COLLECTION_CLUSTER_META);
        let mut options = FindOptions::new();
        options.limit = Some(i64::from(limit));
        find_with_options(collection, filter, options, None)
    }

    fn persist_cluster_meta(&self, meta: ClusterMeta) -> Result<()> {
        let filter = doc! {"cluster_id" => &meta.cluster_id};
        let collection = self.client.db(&self.db).collection(COLLECTION_CLUSTER_META);
        let document = bson::to_bson(&meta).with_context(|_| ErrorKind::MongoDBBsonEncode)?;
        let document = match document {
            Bson::Document(document) => document,
            _ => panic!("ClusterMeta failed to encode as BSON document"),
        };
        replace_one(collection, filter, document)
    }

    fn persist_event(&self, event: Event) -> Result<()> {
        let collection = self.client.db(&self.db).collection(COLLECTION_EVENTS);
        let event = EventDocument::from(event);
        let document = bson::to_bson(&event).with_context(|_| ErrorKind::MongoDBBsonEncode)?;
        let document = match document {
            Bson::Document(document) => document,
            _ => panic!("Event failed to encode as BSON document"),
        };
        insert_one(collection, document)
    }

    fn top_clusters(&self) -> Result<Cursor<ClusterMeta>> {
        let filter = doc! {};
        let sort = doc! {
            "shards" => -1,
            "nodes" => -1,
            "cluster_id" => 1,
        };
        let mut options = FindOptions::new();
        options.limit = Some(i64::from(TOP_CLUSTERS_LIMIT));
        options.sort = Some(sort);
        let collection = self.client.db(&self.db).collection(COLLECTION_CLUSTER_META);
        find_with_options(collection, filter, options, None)
    }
}
