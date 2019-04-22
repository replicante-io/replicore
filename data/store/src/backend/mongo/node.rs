use mongodb::db::ThreadedDatabase;
use mongodb::Client;
use mongodb::ThreadedClient;

use replicante_data_models::Node as NodeModel;

use super::super::super::store::node::NodeAttribures;
use super::super::super::Result;
use super::super::NodeInterface;
use super::common::find_one;
use super::constants::COLLECTION_NODES;

/// Node operations implementation using MongoDB.
pub struct Node {
    client: Client,
    db: String,
}

impl Node {
    pub fn new(client: Client, db: String) -> Node {
        Node { client, db }
    }
}

impl NodeInterface for Node {
    fn get(&self, attrs: &NodeAttribures) -> Result<Option<NodeModel>> {
        let filter = doc! {
            "cluster_id" => &attrs.cluster_id,
            "node_id" => &attrs.node_id,
        };
        let collection = self.client.db(&self.db).collection(COLLECTION_NODES);
        find_one(collection, filter)
    }
}
