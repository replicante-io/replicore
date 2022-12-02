use std::sync::Arc;

use failure::Fail;
use failure::ResultExt;
use mongodb::bson::doc;
use mongodb::sync::Client;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;

use replicante_externals_mongodb::operations::find;
use replicante_models_core::scope::Namespace;

use super::super::NamespacesInterface;
use super::constants::COLLECTION_NAMESPACES;
use crate::Cursor;
use crate::ErrorKind;
use crate::Result;

/// Namespaces operations implementation using MongoDB.
pub struct Namespaces {
    client: Client,
    db: String,
    tracer: Option<Arc<Tracer>>,
}

impl Namespaces {
    pub fn new<T>(client: Client, db: String, tracer: T) -> Namespaces
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let tracer = tracer.into();
        Namespaces { client, db, tracer }
    }
}

impl NamespacesInterface for Namespaces {
    fn iter(&self, span: Option<SpanContext>) -> Result<Cursor<Namespace>> {
        let filter = doc! {};
        let collection = self.client.database(&self.db).collection(COLLECTION_NAMESPACES);
        let cursor = find(collection, filter, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?
            .map(|item| item.map_err(|error| error.context(ErrorKind::MongoDBCursor).into()));
        Ok(Cursor::new(cursor))
    }
}
