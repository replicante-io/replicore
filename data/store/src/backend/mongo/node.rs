use std::ops::Deref;
use std::sync::Arc;

use bson::bson;
use bson::doc;
use mongodb::db::ThreadedDatabase;
use mongodb::Client;
use mongodb::ThreadedClient;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;

use replicante_data_models::Node as NodeModel;

use super::super::super::store::node::NodeAttribures;
use super::super::super::Result;
use super::super::NodeInterface;
use super::common::find_one;
use super::constants::COLLECTION_NODES;
use super::document::NodeDocument;

/// Node operations implementation using MongoDB.
pub struct Node {
    client: Client,
    db: String,
    tracer: Option<Arc<Tracer>>,
}

impl Node {
    pub fn new<T>(client: Client, db: String, tracer: T) -> Node
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let tracer = tracer.into();
        Node { client, db, tracer }
    }
}

impl NodeInterface for Node {
    fn get(&self, attrs: &NodeAttribures, span: Option<SpanContext>) -> Result<Option<NodeModel>> {
        let filter = doc! {
            "cluster_id" => &attrs.cluster_id,
            "node_id" => &attrs.node_id,
        };
        let collection = self.client.db(&self.db).collection(COLLECTION_NODES);
        let document: Option<NodeDocument> = find_one(
            collection,
            filter,
            span,
            self.tracer.as_ref().map(|tracer| tracer.deref()),
        )?;
        Ok(document.map(NodeModel::from))
    }
}
