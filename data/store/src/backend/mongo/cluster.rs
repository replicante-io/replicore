use mongodb::db::ThreadedDatabase;
use mongodb::Client;
use mongodb::ThreadedClient;

use replicante_data_models::ClusterDiscovery;

use super::super::super::store::cluster::ClusterAttribures;
use super::super::super::Result;
use super::super::ClusterInterface;
use super::common::find_one;
use super::constants::COLLECTION_DISCOVERIES;

/// Clusters operations implementation using MongoDB.
pub struct Cluster {
    client: Client,
    db: String,
}

impl Cluster {
    pub fn new(client: Client, db: String) -> Cluster {
        Cluster { client, db }
    }
}

impl ClusterInterface for Cluster {
    fn discovery(&self, attrs: &ClusterAttribures) -> Result<Option<ClusterDiscovery>> {
        let filter = doc! {"cluster_id" => &attrs.cluster_id};
        let collection = self.client.db(&self.db).collection(COLLECTION_DISCOVERIES);
        find_one(collection, filter)
    }
}
