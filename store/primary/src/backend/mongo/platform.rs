use std::sync::Arc;

use failure::ResultExt;
use mongodb::bson::doc;
use mongodb::sync::Client;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;
use replisdk::core::models::platform::Platform as PlatformModel;

use replicante_externals_mongodb::operations::find_one;

use super::super::PlatformInterface;
use super::constants::COLLECTION_PLATFORMS;
use crate::store::platform::PlatformAttributes;
use crate::ErrorKind;
use crate::Result;

/// Platform operations implementation using MongoDB.
pub struct Platform {
    client: Client,
    db: String,
    tracer: Option<Arc<Tracer>>,
}

impl Platform {
    pub fn new<T>(client: Client, db: String, tracer: T) -> Platform
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let tracer = tracer.into();
        Platform { client, db, tracer }
    }
}

impl PlatformInterface for Platform {
    fn get(
        &self,
        attrs: &PlatformAttributes,
        span: Option<SpanContext>,
    ) -> Result<Option<PlatformModel>> {
        let filter = doc! {
            "ns_id": &attrs.ns_id,
            "name": &attrs.platform_id,
        };
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_PLATFORMS);
        let document = find_one(collection, filter, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(document)
    }
}
