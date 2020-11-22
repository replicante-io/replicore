use std::collections::HashSet;
use std::sync::Arc;

use bson::doc;
use bson::Bson;
use failure::Fail;
use failure::ResultExt;
use mongodb::sync::Client;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;

use replicante_externals_mongodb::operations::aggregate;
use replicante_externals_mongodb::operations::find;
use replicante_models_core::agent::Node;

use super::super::NodesInterface;
use super::constants::COLLECTION_NODES;
use super::document::NodeDocument;
use crate::store::nodes::NodesAttribures;
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
    fn iter(&self, attrs: &NodesAttribures, span: Option<SpanContext>) -> Result<Cursor<Node>> {
        let filter = doc! {"cluster_id": &attrs.cluster_id};
        let collection = self.client.database(&self.db).collection(COLLECTION_NODES);
        let cursor = find(collection, filter, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?
            .map(|item| item.map_err(|error| error.context(ErrorKind::MongoDBCursor).into()))
            .map(|result: Result<NodeDocument>| result.map(Node::from));
        Ok(Cursor::new(cursor))
    }

    fn kinds(&self, attrs: &NodesAttribures, span: Option<SpanContext>) -> Result<HashSet<String>> {
        // Let mongo figure out the kinds with an aggregation.
        let filter = doc! {"$match": {
            "cluster_id": &attrs.cluster_id,
            "stale": false,
        }};
        let group = doc! {"$group": {
            "_id": "$cluster_id",
            "kinds": {"$addToSet": "$kind"},
        }};
        let pipeline = vec![filter, group];

        // Run aggrgation and grab the one and only (expected) result.
        let collection = self.client.database(&self.db).collection(COLLECTION_NODES);
        let mut cursor = aggregate(collection, pipeline, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        let kinds: Bson = match cursor.next() {
            None => return Ok(HashSet::new()),
            Some(kinds) => kinds.with_context(|_| ErrorKind::MongoDBCursor)?,
        };
        if cursor.next().is_some() {
            return Err(ErrorKind::DuplicateRecord(
                "aggregated cluster kinds",
                attrs.cluster_id.clone(),
            )
            .into());
        }
        let kinds = kinds
            .as_document()
            .ok_or_else(|| ErrorKind::InvalidRecord(attrs.cluster_id.clone()))?
            .get("kinds")
            .ok_or_else(|| ErrorKind::InvalidRecord(attrs.cluster_id.clone()))?
            .clone();
        let kinds: HashSet<String> = bson::from_bson(kinds)
            .with_context(|_| ErrorKind::InvalidRecord(attrs.cluster_id.clone()))?;
        Ok(kinds)
    }
}
