use std::collections::HashMap;

use bson::Document;
use error_chain::ChainedError;

use mongodb::Client;
use mongodb::ThreadedClient;
use mongodb::db::ThreadedDatabase;

use super::Result;
use super::ResultExt;
use super::ValidationResult;


use super::constants::COLLECTION_EVENTS;
use super::constants::EXPECTED_COLLECTIONS;

use super::metrics::MONGODB_OPS_COUNT;
use super::metrics::MONGODB_OPS_DURATION;
use super::metrics::MONGODB_OP_ERRORS_COUNT;


const GROUP_PERF_INDEX: &'static str = "perf/index";
const GROUP_STORE_ERROR: &'static str = "store/error";
const GROUP_STORE_MISSING: &'static str = "store/missing";
const GROUP_STORE_SCHEMA: &'static str = "store/schema";


/// Extra information about a collection.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
struct CollectionInfo {
    pub capped: bool,
    pub kind: String,
    pub read_only: bool,
}


/// Subset of the validator looking for needed collections and their configuration.
pub struct SchemaValidator {
    client: Client,
    db: String,
}

impl SchemaValidator {
    pub fn new(db: String, client: Client) -> SchemaValidator {
        SchemaValidator { client, db }
    }

    /// Check for the existence of the needed collections.
    ///
    /// Also looks to see if the `events` collection is capped or TTL indexed.
    pub fn schema(&self) -> Result<Vec<ValidationResult>> {
        let collections = self.collections()
            .chain_err(|| "Failed to list collections")?;
        let mut results = Vec::new();

        // Check all needed collections exist and are writable.
        for collection in EXPECTED_COLLECTIONS.iter() {
            let name: &'static str = collection.clone();
            let collection = match collections.get(name) {
                Some(info) => info,
                None => {
                    results.push(ValidationResult::error(
                        name, format!("needed collection '{}' not found", name),
                        GROUP_STORE_MISSING
                    ));
                    continue;
                }
            };
            if collection.kind != "collection" {
                results.push(ValidationResult::error(
                    name,
                    format!("need collection '{}', but found a '{}'", name, collection.kind),
                    GROUP_STORE_SCHEMA
                ));
            }
            if collection.read_only {
                results.push(ValidationResult::error(
                    name, format!("need collection '{}' to be writable", name), GROUP_STORE_SCHEMA
                ));
            }
        }

        // Check `events` collection is capped or TTL indexed.
        if let Some(collection) = collections.get(COLLECTION_EVENTS) {
            let capped = collection.capped;
            match self.has_ttl_index(COLLECTION_EVENTS) {
                Err(error) => {
                    let error = error.display_chain().to_string();
                    results.push(ValidationResult::result(
                        COLLECTION_EVENTS, format!("failed to check indexes: {}", error),
                        GROUP_STORE_ERROR
                    ));
                },
                Ok(ttled) => {
                    if !(capped || ttled) {
                        results.push(ValidationResult::result(
                            COLLECTION_EVENTS,
                            "events collection should be capped or have a TTL index",
                            GROUP_PERF_INDEX
                        ));
                    }
                }
            };
        }

        Ok(results)
    }
}

impl SchemaValidator {
    /// List all collections in the database.
    fn collections(&self) -> Result<HashMap<String, CollectionInfo>> {
        let mut collections = HashMap::new();
        let db = self.client.db(&self.db);
        MONGODB_OPS_COUNT.with_label_values(&["listCollections"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["listCollections"]).start_timer();
        let cursor = db.list_collections(None).map_err(|error| {
            MONGODB_OP_ERRORS_COUNT.with_label_values(&["listCollections"]).inc();
            error
        })?;
        for collection in cursor {
            let collection = collection?;
            let name = collection
                .get_str("name").expect("Unable to determine collection name")
                .into();
            let kind = collection
                .get_str("type").expect("Unable to determine collecton type")
                .into();
            let capped = collection
                .get_document("options").expect("Unable to get collection options")
                .get_bool("capped").unwrap_or(false);
            let read_only = collection
                .get_document("info").expect("Unable to get collection info")
                .get_bool("readOnly").expect("Unable to determine if collection is read only");
            collections.insert(name, CollectionInfo {
                capped,
                kind,
                read_only,
            });
        }
        Ok(collections)
    }

    /// Check a collection for the presence of a TTL index.
    fn has_ttl_index(&self, collection: &'static str) -> Result<bool> {
        let indexes = self.indexes(collection)?;
        for index in indexes {
            if let Ok(options) = index.get_document("options") {
                if options.get("expireAfterSeconds").is_some() {
                    return Ok(true);
                }
            };
        }
        Ok(false)
    }

    /// List indexes on a collection.
    fn indexes(&self, collection: &'static str) -> Result<Vec<Document>> {
        let db = self.client.db(&self.db);
        let collection = db.collection(collection);
        let mut indexes = Vec::new();
        MONGODB_OPS_COUNT.with_label_values(&["listIndexes"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["listIndexes"]).start_timer();
        let cursor = collection.list_indexes().map_err(|error| {
            MONGODB_OP_ERRORS_COUNT.with_label_values(&["listIndexes"]).inc();
            error
        })?;
        for index in cursor {
            indexes.push(index?);
        }
        Ok(indexes)
    }
}
