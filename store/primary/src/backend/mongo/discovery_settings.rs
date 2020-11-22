use std::sync::Arc;

use bson::doc;
use failure::ResultExt;
use mongodb::options::FindOptions;
use mongodb::sync::Client;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;

use replicante_externals_mongodb::operations::delete_one;
use replicante_externals_mongodb::operations::find_with_options;

use super::super::DiscoverySettingsInterface;
use super::constants::COLLECTION_DISCOVERY_SETTINGS;
use super::document::DiscoverySettingsDocument;
use crate::store::discovery_settings::DiscoverySettingsAttributes;
use crate::Cursor;
use crate::ErrorKind;
use crate::Result;

/// Discovery settings operations implementation using MongoDB.
pub struct DiscoverySettings {
    client: Client,
    db: String,
    tracer: Option<Arc<Tracer>>,
}

impl DiscoverySettings {
    pub fn new<T>(client: Client, db: String, tracer: T) -> DiscoverySettings
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let tracer = tracer.into();
        DiscoverySettings { client, db, tracer }
    }
}

impl DiscoverySettingsInterface for DiscoverySettings {
    fn delete(
        &self,
        attrs: &DiscoverySettingsAttributes,
        name: &str,
        span: Option<SpanContext>,
    ) -> Result<()> {
        let filter = doc! {
            "namespace": &attrs.namespace,
            "name": name,
        };
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_DISCOVERY_SETTINGS);
        delete_one(collection, filter, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(())
    }

    fn iter_names(
        &self,
        attrs: &DiscoverySettingsAttributes,
        span: Option<SpanContext>,
    ) -> Result<Cursor<String>> {
        let filter = doc! {"namespace": &attrs.namespace};
        let mut options = FindOptions::default();
        options.sort = Some(doc! {"name": 1});
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_DISCOVERY_SETTINGS);
        let cursor = find_with_options(collection, filter, options, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        let cursor = cursor.map(|document| {
            let document: DiscoverySettingsDocument =
                document.with_context(|_| ErrorKind::MongoDBCursor)?;
            Ok(document.settings.name)
        });
        Ok(Cursor::new(cursor))
    }
}
