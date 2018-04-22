use bson;
use bson::Bson;

use mongodb::Client;
use mongodb::ThreadedClient;
use mongodb::coll::Collection;
use mongodb::coll::options::FindOneAndUpdateOptions;
use mongodb::coll::options::FindOptions;
use mongodb::db::ThreadedDatabase;

use regex;

use replicante_data_models::ClusterDiscovery;
use replicante_data_models::ClusterMeta;

use super::super::super::Result;
use super::super::super::ResultExt;

use super::constants::COLLECTION_CLUSTER_META;
use super::constants::COLLECTION_DISCOVERIES;

use super::constants::FAIL_FIND_CLUSTERS;
use super::constants::FAIL_FIND_CLUSTER_DISCOVERY;
use super::constants::FAIL_FIND_CLUSTER_META;

use super::constants::FAIL_PERSIST_CLUSTER_DISCOVERY;
use super::constants::FAIL_PERSIST_CLUSTER_META;

use super::constants::FAIL_TOP_CLUSTERS;
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

    pub fn cluster_discovery(&self, cluster: String) -> Result<ClusterDiscovery> {
        let filter = doc!{"name" => cluster};
        MONGODB_OPS_COUNT.with_label_values(&["findOne"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["findOne"]).start_timer();
        let collection = self.collection_discoveries();
        let discovery = collection.find_one(Some(filter), None)
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["findOne"]).inc();
                error
            })
            .chain_err(|| FAIL_FIND_CLUSTER_DISCOVERY)?;
        let discovery: Result<_> = discovery.ok_or("Cluster not found".into());
        let discovery = discovery.chain_err(|| FAIL_FIND_CLUSTER_DISCOVERY)?;
        let discovery = bson::from_bson::<ClusterDiscovery>(bson::Bson::Document(discovery))
            .chain_err(|| FAIL_FIND_CLUSTER_DISCOVERY)?;
        Ok(discovery)
    }

    pub fn cluster_meta(&self, cluster: String) -> Result<ClusterMeta> {
        let filter = doc!{"name" => cluster};
        MONGODB_OPS_COUNT.with_label_values(&["findOne"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["findOne"]).start_timer();
        let collection = self.collection_cluster_meta();
        let meta = collection.find_one(Some(filter), None)
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["findOne"]).inc();
                error
            })
            .chain_err(|| FAIL_FIND_CLUSTER_META)?;
        let meta: Result<_> = meta.ok_or("Cluster not found".into());
        let meta = meta.chain_err(|| FAIL_FIND_CLUSTER_META)?;
        let meta = bson::from_bson::<ClusterMeta>(bson::Bson::Document(meta))
            .chain_err(|| FAIL_FIND_CLUSTER_META)?;
        Ok(meta)
    }

    pub fn find_clusters(&self, search: String, limit: u8) -> Result<Vec<ClusterMeta>> {
        let search = regex::escape(&search);
        let filter = doc!{"name" => {"$regex" => search, "$options" => "i"}};
        let mut options = FindOptions::new();
        options.limit = Some(limit as i64);

        MONGODB_OPS_COUNT.with_label_values(&["find"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["find"]).start_timer();
        let collection = self.collection_cluster_meta();
        let cursor = collection.find(Some(filter), Some(options))
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["find"]).inc();
                error
            })
            .chain_err(|| FAIL_FIND_CLUSTERS)?;

        let mut clusters = Vec::new();
        for doc in cursor {
            let doc = doc.chain_err(|| FAIL_FIND_CLUSTERS)?;
            let cluster = bson::from_bson::<ClusterMeta>(bson::Bson::Document(doc))
                .chain_err(|| FAIL_FIND_CLUSTERS)?;
            clusters.push(cluster);
        }
        Ok(clusters)
    }

    pub fn top_clusters(&self) -> Result<Vec<ClusterMeta>> {
        let sort = doc!{
            "nodes" => -1,
            "name" => 1,
        };
        let mut options = FindOptions::new();
        options.limit = Some(TOP_CLUSTERS_LIMIT as i64);
        options.sort = Some(sort);
        let collection = self.collection_cluster_meta();
        MONGODB_OPS_COUNT.with_label_values(&["find"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["find"]).start_timer();
        let cursor = collection.find(None, Some(options))
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["find"]).inc();
                error
            })
            .chain_err(|| FAIL_TOP_CLUSTERS)?;

        let mut clusters = Vec::new();
        for doc in cursor {
            let doc = doc.chain_err(|| FAIL_TOP_CLUSTERS)?;
            let cluster = bson::from_bson::<ClusterMeta>(bson::Bson::Document(doc))
                .chain_err(|| FAIL_TOP_CLUSTERS)?;
            clusters.push(cluster);
        }
        Ok(clusters)
    }

    pub fn persist_cluster_meta(&self, meta: ClusterMeta) -> Result<Option<ClusterMeta>> {
        let replacement = bson::to_bson(&meta).chain_err(|| FAIL_PERSIST_CLUSTER_META)?;
        let replacement = match replacement {
            Bson::Document(replacement) => replacement,
            _ => panic!("ClusterMeta failed to encode as BSON document")
        };
        let filter = doc!{"name" => meta.name};
        let collection = self.collection_cluster_meta();
        let mut options = FindOneAndUpdateOptions::new();
        options.upsert = Some(true);
        MONGODB_OPS_COUNT.with_label_values(&["findOneAndReplace"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["findOneAndReplace"]).start_timer();
        let old = collection.find_one_and_replace(filter, replacement, Some(options))
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["findOneAndReplace"]).inc();
                error
            })
            .chain_err(|| FAIL_PERSIST_CLUSTER_META)?;
        match old {
            None => Ok(None),
            Some(doc) => {
                let meta = bson::from_bson::<ClusterMeta>(bson::Bson::Document(doc))
                    .chain_err(|| FAIL_PERSIST_CLUSTER_META)?;
                Ok(Some(meta))
            }
        }
    }

    pub fn persist_discovery(&self, cluster: ClusterDiscovery) -> Result<Option<ClusterDiscovery>> {
        let replacement = bson::to_bson(&cluster).chain_err(|| FAIL_PERSIST_CLUSTER_DISCOVERY)?;
        let replacement = match replacement {
            Bson::Document(replacement) => replacement,
            _ => panic!("ClusterDiscovery failed to encode as BSON document")
        };
        let filter = doc!{"name" => cluster.name};
        let collection = self.collection_discoveries();
        let mut options = FindOneAndUpdateOptions::new();
        options.upsert = Some(true);
        MONGODB_OPS_COUNT.with_label_values(&["findOneAndReplace"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["findOneAndReplace"]).start_timer();
        let old = collection.find_one_and_replace(filter, replacement, Some(options))
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["findOneAndReplace"]).inc();
                error
            })
            .chain_err(|| FAIL_PERSIST_CLUSTER_DISCOVERY)?;
        match old {
            None => Ok(None),
            Some(doc) => {
                let discovery = bson::from_bson::<ClusterDiscovery>(bson::Bson::Document(doc))
                    .chain_err(|| FAIL_PERSIST_CLUSTER_DISCOVERY)?;
                Ok(Some(discovery))
            }
        }
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
