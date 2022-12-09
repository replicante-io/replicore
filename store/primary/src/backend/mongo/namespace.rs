use std::sync::Arc;

use failure::ResultExt;
use mongodb::bson::doc;
use mongodb::sync::Client;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;

use replicante_externals_mongodb::operations::find_one;
use replicante_models_core::scope::Namespace as NamespaceModel;

use super::super::NamespaceInterface;
use super::constants::COLLECTION_NAMESPACES;
use crate::store::namespace::NamespaceAttributes;
use crate::ErrorKind;
use crate::Result;

/// Namespace operations implementation using MongoDB.
pub struct Namespace {
    client: Client,
    db: String,
    tracer: Option<Arc<Tracer>>,
}

impl Namespace {
    pub fn new<T>(client: Client, db: String, tracer: T) -> Namespace
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let tracer = tracer.into();
        Namespace { client, db, tracer }
    }
}

impl NamespaceInterface for Namespace {
    fn get(
        &self,
        attrs: &NamespaceAttributes,
        span: Option<SpanContext>,
    ) -> Result<Option<NamespaceModel>> {
        let filter = doc! { "ns_id": &attrs.ns_id };
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_NAMESPACES);
        let document = find_one(collection, filter, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(document)
    }
}
