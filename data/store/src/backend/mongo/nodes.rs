use mongodb::db::ThreadedDatabase;
use mongodb::Client;
use mongodb::ThreadedClient;

use replicante_data_models::Node;

use super::super::super::store::nodes::NodesAttribures;
use super::super::super::Cursor;
use super::super::super::Result;
use super::super::NodesInterface;
use super::common::find;
use super::constants::COLLECTION_NODES;
use super::document::NodeDocument;

/// Nodes operations implementation using MongoDB.
pub struct Nodes {
    client: Client,
    db: String,
}

impl Nodes {
    pub fn new(client: Client, db: String) -> Nodes {
        Nodes { client, db }
    }
}

impl NodesInterface for Nodes {
    fn iter(&self, attrs: &NodesAttribures) -> Result<Cursor<Node>> {
        let filter = doc! {"cluster_id" => &attrs.cluster_id};
        let collection = self.client.db(&self.db).collection(COLLECTION_NODES);
        let cursor = find(collection, filter)?
            .map(|result: Result<NodeDocument>| result.map(Node::from));
        Ok(Cursor(Box::new(cursor)))
    }
}
