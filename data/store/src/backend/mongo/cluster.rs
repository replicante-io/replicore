use bson;
use bson::Bson;
use failure::ResultExt;
use mongodb::Client;
use mongodb::ThreadedClient;
use mongodb::coll::Collection;
use mongodb::coll::options::FindOptions;
use mongodb::coll::options::UpdateOptions;
use mongodb::db::ThreadedDatabase;
use regex;

use replicante_data_models::ClusterDiscovery;
use replicante_data_models::ClusterMeta;

use super::super::super::ErrorKind;
use super::super::super::Result;

use super::constants::COLLECTION_CLUSTER_META;
use super::constants::COLLECTION_DISCOVERIES;

use super::constants::TOP_CLUSTERS_LIMIT;

use super::metrics::MONGODB_OPS_COUNT;
use super::metrics::MONGODB_OPS_DURATION;
use super::metrics::MONGODB_OP_ERRORS_COUNT;


/// Subset of the `Store` trait that deals with clusters.
pub struct ClusterStore {
    client: Client,
    db: String,
}

impl ClusterStore {
    pub fn new(client: Client, db: String) -> ClusterStore {
        ClusterStore { client, db }
    }

    pub fn cluster_discovery(&self, cluster_id: String) -> Result<Option<ClusterDiscovery>> {
        let filter = doc!{"cluster_id" => cluster_id};
        MONGODB_OPS_COUNT.with_label_values(&["findOne"]).inc();
        let timer = MONGODB_OPS_DURATION.with_label_values(&["findOne"]).start_timer();
        let collection = self.collection_discoveries();
        let discovery = collection.find_one(Some(filter), None)
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["findOne"]).inc();
                error
            })
            .with_context(|_| ErrorKind::MongoDBOperation("findOne"))?;
        timer.observe_duration();
        if discovery.is_none() {
            return Ok(None);
        }
        let discovery = discovery.unwrap();
        let discovery = bson::from_bson::<ClusterDiscovery>(bson::Bson::Document(discovery))
            .with_context(|_| ErrorKind::MongoDBBsonDecode)?;
        Ok(Some(discovery))
    }

    pub fn cluster_meta(&self, cluster_id: String) -> Result<Option<ClusterMeta>> {
        let filter = doc!{"cluster_id" => cluster_id};
        MONGODB_OPS_COUNT.with_label_values(&["findOne"]).inc();
        let timer = MONGODB_OPS_DURATION.with_label_values(&["findOne"]).start_timer();
        let collection = self.collection_cluster_meta();
        let meta = collection.find_one(Some(filter), None)
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["findOne"]).inc();
                error
            })
            .with_context(|_| ErrorKind::MongoDBOperation("findOne"))?;
        timer.observe_duration();
        if meta.is_none() {
            return Ok(None);
        }
        let meta = meta.unwrap();
        let meta = bson::from_bson::<ClusterMeta>(bson::Bson::Document(meta))
            .with_context(|_| ErrorKind::MongoDBBsonDecode)?;
        Ok(Some(meta))
    }

    pub fn find_clusters(&self, search: &str, limit: u8) -> Result<Vec<ClusterMeta>> {
        let search = regex::escape(&search);
        let filter = doc!{"cluster_id" => {"$regex" => search, "$options" => "i"}};
        let mut options = FindOptions::new();
        options.limit = Some(i64::from(limit));

        MONGODB_OPS_COUNT.with_label_values(&["find"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["find"]).start_timer();
        let collection = self.collection_cluster_meta();
        let cursor = collection.find(Some(filter), Some(options))
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["find"]).inc();
                error
            })
            .with_context(|_| ErrorKind::MongoDBOperation("find"))?;
        let mut clusters = Vec::new();
        for doc in cursor {
            let doc = doc.with_context(|_| ErrorKind::MongoDBCursor("find"))?;
            let cluster = bson::from_bson::<ClusterMeta>(bson::Bson::Document(doc))
                .with_context(|_| ErrorKind::MongoDBBsonDecode)?;
            clusters.push(cluster);
        }
        Ok(clusters)
    }

    pub fn top_clusters(&self) -> Result<Vec<ClusterMeta>> {
        let sort = doc!{
            "nodes" => -1,
            "cluster_id" => 1,
        };
        let mut options = FindOptions::new();
        options.limit = Some(i64::from(TOP_CLUSTERS_LIMIT));
        options.sort = Some(sort);
        let collection = self.collection_cluster_meta();
        MONGODB_OPS_COUNT.with_label_values(&["find"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["find"]).start_timer();
        let cursor = collection.find(None, Some(options))
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["find"]).inc();
                error
            })
            .with_context(|_| ErrorKind::MongoDBOperation("find"))?;
        let mut clusters = Vec::new();
        for doc in cursor {
            let doc = doc.with_context(|_| ErrorKind::MongoDBCursor("find"))?;
            let cluster = bson::from_bson::<ClusterMeta>(bson::Bson::Document(doc))
                .with_context(|_| ErrorKind::MongoDBBsonDecode)?;
            clusters.push(cluster);
        }
        Ok(clusters)
    }

    pub fn persist_cluster_meta(&self, meta: ClusterMeta) -> Result<()> {
        let replacement = bson::to_bson(&meta).with_context(|_| ErrorKind::MongoDBBsonEncode)?;
        let replacement = match replacement {
            Bson::Document(replacement) => replacement,
            _ => panic!("ClusterMeta failed to encode as BSON document")
        };
        let filter = doc!{"cluster_id" => meta.cluster_id};
        let collection = self.collection_cluster_meta();
        let mut options = UpdateOptions::new();
        options.upsert = Some(true);
        MONGODB_OPS_COUNT.with_label_values(&["replaceOne"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["replaceOne"]).start_timer();
        collection.replace_one(filter, replacement, Some(options))
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["replaceOne"]).inc();
                error
            })
            .with_context(|_| ErrorKind::MongoDBOperation("replaceOne"))?;
        Ok(())
    }

    pub fn persist_discovery(&self, cluster: ClusterDiscovery) -> Result<()> {
        let replacement = bson::to_bson(&cluster).with_context(|_| ErrorKind::MongoDBBsonEncode)?;
        let replacement = match replacement {
            Bson::Document(replacement) => replacement,
            _ => panic!("ClusterDiscovery failed to encode as BSON document")
        };
        let filter = doc!{"cluster_id" => cluster.cluster_id.clone()};
        let collection = self.collection_discoveries();
        let mut options = UpdateOptions::new();
        options.upsert = Some(true);
        MONGODB_OPS_COUNT.with_label_values(&["replaceOne"]).inc();
        let timer = MONGODB_OPS_DURATION.with_label_values(&["replaceOne"]).start_timer();
        collection.replace_one(filter, replacement, Some(options))
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["replaceOne"]).inc();
                error
            })
            .with_context(|_| ErrorKind::MongoDBOperation("replaceOne"))?;
        timer.observe_duration();
        Ok(())
    }

    /// Returns the `clusters_meta` collection.
    fn collection_cluster_meta(&self) -> Collection {
        self.client.db(&self.db).collection(COLLECTION_CLUSTER_META)
    }

    /// Returns the `discoveries` collection.
    fn collection_discoveries(&self) -> Collection {
        self.client.db(&self.db).collection(COLLECTION_DISCOVERIES)
    }
}
