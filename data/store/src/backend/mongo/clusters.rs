use bson;
use bson::Bson;

use mongodb::Client;
use mongodb::ThreadedClient;
use mongodb::coll::Collection;
use mongodb::coll::options::FindOneAndUpdateOptions;
use mongodb::coll::options::FindOptions;
use mongodb::db::ThreadedDatabase;

use regex;

use replicante_data_models::Cluster;
use replicante_data_models::webui::ClusterListItem;

use super::super::super::Result;
use super::super::super::ResultExt;

use super::constants::COLLECTION_CLUSTERS;
use super::constants::COLLECTION_CLUSTER_LIST;
use super::constants::COLLECTION_NODES;
use super::constants::FAIL_CLUSTER_LIST_REBUILD;
use super::constants::FAIL_FIND_CLUSTERS;
use super::constants::FAIL_PERSIST_CLUSTER;
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

    pub fn fetch_top_clusters(&self) -> Result<Vec<ClusterListItem>> {
        let sort = doc!{
            "name" => 1,
            "nodes" => -1
        };
        let mut options = FindOptions::new();
        options.limit = Some(TOP_CLUSTERS_LIMIT as i64);
        options.sort = Some(sort);
        let collection = self.collection_cluster_lists();
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
            let cluster = bson::from_bson::<ClusterListItem>(bson::Bson::Document(doc))
                .chain_err(|| FAIL_TOP_CLUSTERS)?;
            clusters.push(cluster);
        }
        Ok(clusters)
    }

    pub fn find_clusters(&self, search: String, limit: u8) -> Result<Vec<ClusterListItem>> {
        let search = regex::escape(&search);
        let filter = doc!{"name" => {"$regex" => search, "$options" => "i"}};
        let mut options = FindOptions::new();
        options.limit = Some(limit as i64);

        MONGODB_OPS_COUNT.with_label_values(&["find"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["find"]).start_timer();
        let collection = self.collection_cluster_lists();
        let cursor = collection.find(Some(filter), Some(options))
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["find"]).inc();
                error
            })
            .chain_err(|| FAIL_FIND_CLUSTERS)?;

        let mut clusters = Vec::new();
        for doc in cursor {
            let doc = doc.chain_err(|| FAIL_FIND_CLUSTERS)?;
            let cluster = bson::from_bson::<ClusterListItem>(bson::Bson::Document(doc))
                .chain_err(|| FAIL_FIND_CLUSTERS)?;
            clusters.push(cluster);
        }
        Ok(clusters)
    }

    pub fn persist_cluster(&self, cluster: Cluster) -> Result<Option<Cluster>> {
        let replacement = bson::to_bson(&cluster).chain_err(|| FAIL_PERSIST_CLUSTER)?;
        let replacement = match replacement {
            Bson::Document(replacement) => replacement,
            _ => panic!("Cluster failed to encode as BSON document")
        };
        let filter = doc!{"name" => cluster.name};
        let collection = self.collection_clusters();
        let mut options = FindOneAndUpdateOptions::new();
        options.upsert = Some(true);
        MONGODB_OPS_COUNT.with_label_values(&["findOneAndReplace"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["findOneAndReplace"]).start_timer();
        let old = collection.find_one_and_replace(filter, replacement, Some(options))
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["findOneAndReplace"]).inc();
                error
            })
            .chain_err(|| FAIL_PERSIST_CLUSTER)?;
        self.rebuild_cluster_lists().chain_err(|| FAIL_PERSIST_CLUSTER)?;
        match old {
            None => Ok(None),
            Some(doc) => Ok(Some(bson::from_bson::<Cluster>(Bson::Document(doc))?))
        }
    }

    /// Returns the `cluster_lists` collection.
    fn collection_cluster_lists(&self) -> Collection {
        self.client.db(&self.db).collection(COLLECTION_CLUSTER_LIST)
    }

    /// Returns the `clusters` collection.
    fn collection_clusters(&self) -> Collection {
        self.client.db(&self.db).collection(COLLECTION_CLUSTERS)
    }

    /// Returns the `nodes` collection.
    fn collection_nodes(&self) -> Collection {
        self.client.db(&self.db).collection(COLLECTION_NODES)
    }

    /// Runs an aggregation that rebuilds the `cluster_lists` collection.
    fn rebuild_cluster_lists(&self) -> Result<()> {
        let group = doc! {
            "$group" => {
                "_id" => "$info.datastore.cluster",
                "nodes" => {"$sum" => 1},
                "kinds" => {"$addToSet" => "$info.datastore.kind"},
            }
        };
        let project = doc! {
            "$project" => {
                "name" => "$_id",
                "kinds" => 1,
                "nodes" => 1,
            }
        };
        let out = doc! {"$out" => COLLECTION_CLUSTER_LIST};
        let collection = self.collection_nodes();
        MONGODB_OPS_COUNT.with_label_values(&["aggregate"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["aggregate"]).start_timer();
        collection.aggregate(vec![group, project, out], None)
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["aggregate"]).inc();
                error
            })
            .chain_err(|| FAIL_CLUSTER_LIST_REBUILD)?;
        Ok(())
    }
}
