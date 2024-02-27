use std::sync::Arc;

use failure::Fail;
use failure::ResultExt;
use mongodb::bson::doc;
use mongodb::sync::Client;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;
use replisdk::core::models::platform::Platform as PlatformModel;

use replicante_externals_mongodb::operations::find;

use super::super::PlatformsInterface;
use super::constants::COLLECTION_PLATFORMS;
use crate::store::platforms::PlatformsAttributes;
use crate::Cursor;
use crate::ErrorKind;
use crate::Result;

/// Platforms operations implementation using MongoDB.
pub struct Platforms {
    client: Client,
    db: String,
    tracer: Option<Arc<Tracer>>,
}

impl Platforms {
    pub fn new<T>(client: Client, db: String, tracer: T) -> Platforms
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let tracer = tracer.into();
        Platforms { client, db, tracer }
    }
}

impl PlatformsInterface for Platforms {
    fn iter(
        &self,
        attrs: &PlatformsAttributes,
        span: Option<SpanContext>,
    ) -> Result<Cursor<PlatformModel>> {
        let filter = doc! {"ns_id": &attrs.ns_id};
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_PLATFORMS);
        let cursor = find(collection, filter, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?
            .map(|item| item.map_err(|error| error.context(ErrorKind::MongoDBCursor).into()));
        Ok(Cursor::new(cursor))
    }
}
