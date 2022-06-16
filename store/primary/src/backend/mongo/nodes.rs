use std::sync::Arc;

use bson::doc;
use failure::Fail;
use failure::ResultExt;
use mongodb::sync::Client;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;

use replicante_externals_mongodb::operations::find;
use replicante_models_core::agent::Node;

use super::super::NodesInterface;
use super::constants::COLLECTION_NODES;
use crate::store::nodes::NodesAttributes;
use crate::Cursor;
use crate::ErrorKind;
use crate::Result;

/// Nodes operations implementation using MongoDB.
pub struct Nodes {
    client: Client,
    db: String,
    tracer: Option<Arc<Tracer>>,
}

impl Nodes {
    pub fn new<T>(client: Client, db: String, tracer: T) -> Nodes
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let tracer = tracer.into();
        Nodes { client, db, tracer }
    }
}

impl NodesInterface for Nodes {
    fn iter(&self, attrs: &NodesAttributes, span: Option<SpanContext>) -> Result<Cursor<Node>> {
        let filter = doc! {"cluster_id": &attrs.cluster_id};
        let collection = self.client.database(&self.db).collection(COLLECTION_NODES);
        let cursor = find(collection, filter, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?
            .map(|item| item.map_err(|error| error.context(ErrorKind::MongoDBCursor).into()));
        Ok(Cursor::new(cursor))
    }
}
