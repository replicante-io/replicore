use std::collections::HashSet;
use std::ops::Deref;
use std::sync::Arc;

use bson::bson;
use bson::doc;
use bson::Bson;
use failure::ResultExt;
use mongodb::db::ThreadedDatabase;
use mongodb::Client;
use mongodb::ThreadedClient;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;

use replicante_data_models::Node;

use super::super::super::store::nodes::NodesAttribures;
use super::super::super::Cursor;
use super::super::super::ErrorKind;
use super::super::super::Result;
use super::super::NodesInterface;
use super::common::aggregate;
use super::common::find;
use super::constants::COLLECTION_NODES;
use super::document::NodeDocument;

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
        let filter = doc! {"cluster_id" => &attrs.cluster_id};
        let collection = self.client.db(&self.db).collection(COLLECTION_NODES);
        let cursor = find(
            collection,
            filter,
            span,
            self.tracer.as_ref().map(|tracer| tracer.deref()),
        )?
        .map(|result: Result<NodeDocument>| result.map(Node::from));
        Ok(Cursor(Box::new(cursor)))
    }

    fn kinds(&self, attrs: &NodesAttribures, span: Option<SpanContext>) -> Result<HashSet<String>> {
        // Let mongo figure out the kinds with an aggregation.
        let filter = doc! {"$match" => {
            "cluster_id" => &attrs.cluster_id,
            "stale" => false,
        }};
        let group = doc! {"$group" => {
            "_id" => "$cluster_id",
            "kinds" => {"$addToSet": "$kind"},
        }};
        let pipeline = vec![filter, group];

        // Run aggrgation and grab the one and only (expected) result.
        let collection = self.client.db(&self.db).collection(COLLECTION_NODES);
        let mut cursor = aggregate(
            collection,
            pipeline,
            span,
            self.tracer.as_ref().map(|tracer| tracer.deref()),
        )?;
        let kinds: Bson = match cursor.next() {
            None => return Ok(HashSet::new()),
            Some(kinds) => kinds?,
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
