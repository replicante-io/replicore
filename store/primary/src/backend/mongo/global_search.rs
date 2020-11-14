use std::sync::Arc;

use bson::doc;
use chrono::Utc;
use failure::Fail;
use failure::ResultExt;
use mongodb::sync::Client;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;

use replicante_externals_mongodb::operations::find;
use replicante_models_core::cluster::discovery::DiscoverySettings;

use super::super::GlobalSearchInterface;
use super::constants::COLLECTION_DISCOVERY_SETTINGS;
use super::document::DiscoverySettingsDocument;
use crate::Cursor;
use crate::ErrorKind;
use crate::Result;

/// Global search operations implementation using MongoDB.
pub struct GlobalSearch {
    client: Client,
    db: String,
    tracer: Option<Arc<Tracer>>,
}

impl GlobalSearch {
    pub fn new<T>(client: Client, db: String, tracer: T) -> GlobalSearch
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let tracer = tracer.into();
        GlobalSearch { client, db, tracer }
    }
}

impl GlobalSearchInterface for GlobalSearch {
    fn discoveries_to_run(&self, span: Option<SpanContext>) -> Result<Cursor<DiscoverySettings>> {
        let filter = doc! {"$or" => [
            {"next_run" => null},
            {"next_run" => {"$lte" => Utc::now()}},
        ]};
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_DISCOVERY_SETTINGS);
        let cursor = find(collection, filter, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?
            .map(|item| item.map_err(|error| error.context(ErrorKind::MongoDBCursor).into()))
            .map(|result: Result<DiscoverySettingsDocument>| result.map(DiscoverySettings::from));
        Ok(Cursor::new(cursor))
    }
}
