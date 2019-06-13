use std::collections::HashMap;
use std::collections::HashSet;

use bson;
use bson::Bson;
use bson::Document;
use failure::ResultExt;
use mongodb::coll::Collection;
use mongodb::db::Database;
use mongodb::db::ThreadedDatabase;
use mongodb::Client;
use mongodb::ThreadedClient;

use super::super::super::admin::ValidationResult;
use super::super::super::ErrorKind;
use super::super::super::Result;
use super::super::ValidateInterface;
use super::constants::COLLECTION_EVENTS;
use super::constants::VALIDATE_EXPECTED_COLLECTIONS;
use super::constants::VALIDATE_INDEXES_NEEDED;
use super::constants::VALIDATE_INDEXES_SUGGESTED;
use super::metrics::MONGODB_OPS_COUNT;
use super::metrics::MONGODB_OPS_DURATION;
use super::metrics::MONGODB_OP_ERRORS_COUNT;

const GROUP_PERF_INDEX: &str = "perf/index";
const GROUP_STORE_MISSING: &str = "store/missing";
const GROUP_STORE_SCHEMA: &str = "store/schema";

/// Lookup all collections in the database.
fn collections(db: Database) -> Result<HashMap<String, CollectionInfo>> {
    MONGODB_OPS_COUNT
        .with_label_values(&["listCollections"])
        .inc();
    let _timer = MONGODB_OPS_DURATION
        .with_label_values(&["listCollections"])
        .start_timer();
    let cursor = db
        .list_collections(None)
        .map_err(|error| {
            MONGODB_OP_ERRORS_COUNT
                .with_label_values(&["listCollections"])
                .inc();
            error
        })
        .with_context(|_| ErrorKind::MongoDBOperation("listCollections"))?;
    let mut collections = HashMap::new();
    for collection in cursor {
        let collection =
            collection.with_context(|_| ErrorKind::MongoDBCursor("listCollections"))?;
        let name = collection
            .get_str("name")
            .expect("Unable to determine collection name")
            .into();
        let kind = collection
            .get_str("type")
            .expect("Unable to determine collecton type")
            .into();
        let capped = collection
            .get_document("options")
            .expect("Unable to get collection options")
            .get_bool("capped")
            .unwrap_or(false);
        let read_only = collection
            .get_document("info")
            .expect("Unable to get collection info")
            .get_bool("readOnly")
            .expect("Unable to determine if collection is read only");
        collections.insert(
            name,
            CollectionInfo {
                capped,
                kind,
                read_only,
            },
        );
    }
    Ok(collections)
}

/// Lookup indexes on a collection.
fn fetch_indexes(collection: &Collection) -> Result<HashSet<IndexInfo>> {
    let mut indexes = HashSet::new();
    MONGODB_OPS_COUNT.with_label_values(&["listIndexes"]).inc();
    let _timer = MONGODB_OPS_DURATION
        .with_label_values(&["listIndexes"])
        .start_timer();
    let cursor = collection
        .list_indexes()
        .map_err(|error| {
            MONGODB_OP_ERRORS_COUNT
                .with_label_values(&["listIndexes"])
                .inc();
            error
        })
        .with_context(|_| ErrorKind::MongoDBOperation("listIndexes"))?;
    for index in cursor {
        let index = index.with_context(|_| ErrorKind::MongoDBCursor("listIndexes"))?;
        let index = IndexInfo::parse(&index)?;
        indexes.insert(index);
    }
    Ok(indexes)
}

/// Check if at least one index in the collection is a TTL index.
fn has_ttl_index(collection: Collection) -> Result<bool> {
    let indexes = fetch_indexes(&collection)?;
    for index in indexes {
        if index.expires {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Extra information about a collection.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct CollectionInfo {
    pub capped: bool,
    pub kind: String,
    pub read_only: bool,
}

/// Extra information about an index.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct IndexInfo {
    pub expires: bool,
    pub key: Vec<(String, i8)>,
    pub unique: bool,
}

impl IndexInfo {
    pub fn parse(document: &Document) -> Result<IndexInfo> {
        let mut keys = Vec::new();
        for (key, direction) in document
            .get_document("key")
            .expect("index has no key")
            .iter()
        {
            let direction = match *direction {
                Bson::FloatingPoint(f) if (f - 1.0).abs() < 0.1 => 1,
                Bson::FloatingPoint(f) if (f - -1.0).abs() < 0.1 => -1,
                Bson::I32(i) if i == 1 => 1,
                Bson::I32(i) if i == -1 => -1,
                Bson::I64(i) if i == 1 => 1,
                Bson::I64(i) if i == -1 => -1,
                _ => panic!("key direction is not 1 or -1"),
            };
            keys.push((key.clone(), direction));
        }
        let expires = document.get("expireAfterSeconds").is_some();
        let unique = document.get_bool("unique").unwrap_or(false);
        Ok(IndexInfo {
            expires,
            key: keys,
            unique,
        })
    }
}

/// Validation operations implementation using MongoDB.
pub struct Validate {
    client: Client,
    db: String,
}

impl Validate {
    pub fn new(client: Client, db: String) -> Validate {
        Validate { client, db }
    }
}

impl ValidateInterface for Validate {
    fn indexes(&self) -> Result<Vec<ValidationResult>> {
        let mut results = Vec::new();
        for name in VALIDATE_EXPECTED_COLLECTIONS.iter() {
            let collection = self.client.db(&self.db).collection(name);
            let indexes = fetch_indexes(&collection)?;
            let needed = VALIDATE_INDEXES_NEEDED
                .get(name)
                .unwrap_or_else(|| panic!("needed indexes not configured for '{}'", name));
            for index in needed {
                if !indexes.contains(index) {
                    results.push(ValidationResult::error(
                        String::from(*name),
                        format!("missing required index: {:?}", index),
                        GROUP_STORE_MISSING,
                    ));
                }
            }
            let suggested = VALIDATE_INDEXES_SUGGESTED
                .get(name)
                .unwrap_or_else(|| panic!("suggested indexes not configured for '{}'", name));
            for index in suggested {
                if !indexes.contains(index) {
                    results.push(ValidationResult::result(
                        String::from(*name),
                        format!("recommended index not found: {:?}", index),
                        GROUP_PERF_INDEX,
                    ));
                }
            }
        }
        Ok(results)
    }

    fn removed_entities(&self) -> Result<Vec<ValidationResult>> {
        // There is nothing removed yet.
        Ok(vec![])
    }

    fn schema(&self) -> Result<Vec<ValidationResult>> {
        let collections = collections(self.client.db(&self.db))?;
        let mut results = Vec::new();

        // Check all needed collections exist and are writable.
        for name in VALIDATE_EXPECTED_COLLECTIONS.iter() {
            let collection = match collections.get(*name) {
                Some(info) => info,
                None => {
                    results.push(ValidationResult::error(
                        *name,
                        format!("needed collection '{}' not found", name),
                        GROUP_STORE_MISSING,
                    ));
                    continue;
                }
            };
            if collection.kind != "collection" {
                results.push(ValidationResult::error(
                    *name,
                    format!(
                        "need collection '{}', but found a '{}'",
                        name, collection.kind
                    ),
                    GROUP_STORE_SCHEMA,
                ));
            }
            if collection.read_only {
                results.push(ValidationResult::error(
                    *name,
                    format!("need collection '{}' to be writable", name),
                    GROUP_STORE_SCHEMA,
                ));
            }
        }

        // Check `events` collection is capped or TTL indexed.
        if let Some(collection) = collections.get(COLLECTION_EVENTS) {
            let capped = collection.capped;
            let ttled = has_ttl_index(self.client.db(&self.db).collection(COLLECTION_EVENTS))?;
            if !(capped || ttled) {
                results.push(ValidationResult::result(
                    COLLECTION_EVENTS,
                    "events collection should be capped or have a TTL index",
                    GROUP_PERF_INDEX,
                ));
            }
        }
        Ok(results)
    }
}
