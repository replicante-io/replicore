use std::sync::Arc;

use failure::ResultExt;
use mongodb::bson::doc;
use mongodb::sync::Client;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;

use replicante_externals_mongodb::operations::find_one;
use replicante_models_core::agent::Node as NodeModel;

use super::super::NodeInterface;
use super::constants::COLLECTION_NODES;
use crate::store::node::NodeAttributes;
use crate::ErrorKind;
use crate::Result;

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
    fn get(&self, attrs: &NodeAttributes, span: Option<SpanContext>) -> Result<Option<NodeModel>> {
        let filter = doc! {
            "cluster_id": &attrs.cluster_id,
            "node_id": &attrs.node_id,
        };
        let collection = self.client.database(&self.db).collection(COLLECTION_NODES);
        let document = find_one(collection, filter, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(document)
    }
}
