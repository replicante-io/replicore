use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::BuildHasher;

use bson::Bson;
use bson::Document;
use failure::ResultExt;
use mongodb::coll::Collection;
use mongodb::db::Database;
use mongodb::db::ThreadedDatabase;
use mongodb::Client;
use mongodb::ThreadedClient;

use crate::metrics::MONGODB_OPS_COUNT;
use crate::metrics::MONGODB_OPS_DURATION;
use crate::metrics::MONGODB_OP_ERRORS_COUNT;
use crate::ErrorKind;
use crate::Result;

const GROUP_PERF_INDEX: &str = "perf/index";
const GROUP_STORE_MISSING: &str = "store/missing";
const GROUP_STORE_REMOVED: &str = "store/cleanup";
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
        .with_context(|_| ErrorKind::ListCollectionsOp)?;
    let mut collections = HashMap::new();
    for collection in cursor {
        let collection = collection.with_context(|_| ErrorKind::ListCollectionsCursor)?;
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
pub fn fetch_indexes(collection: &Collection) -> Result<HashSet<IndexInfo>> {
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
        .with_context(|_| ErrorKind::ListIndexesOp)?;
    for index in cursor {
        let index = index.with_context(|_| ErrorKind::ListIndexesCursor)?;
        let index = IndexInfo::parse(&index)?;
        indexes.insert(index);
    }
    Ok(indexes)
}

/// Validate the list of indexes in a MongoDB database against an expected set.
pub fn validate_indexes<S1: BuildHasher, S2: BuildHasher, S3: BuildHasher>(
    client: &Client,
    db: &str,
    collections: &HashSet<&'static str, S1>,
    needed_indexes: &HashMap<&'static str, Vec<IndexInfo>, S2>,
    suggested_indexes: &HashMap<&'static str, Vec<IndexInfo>, S3>,
) -> Result<Vec<ValidationResult>> {
    let mut results = Vec::new();
    for name in collections.iter() {
        let collection = client.db(db).collection(name);
        let indexes = fetch_indexes(&collection)?;
        let needed = needed_indexes
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
        let suggested = suggested_indexes
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

/// Validate the list of deprecated collections agains the MongoDB database.
pub fn validate_removed_collections<S: BuildHasher>(
    client: &Client,
    db: &str,
    removed_collections: &HashSet<&'static str, S>,
) -> Result<Vec<ValidationResult>> {
    let collections = collections(client.db(db))?;
    let mut results = Vec::new();
    for name in removed_collections.iter() {
        if collections.get(*name).is_some() {
            results.push(ValidationResult::result(
                *name,
                format!("found removed collection '{}'", name),
                GROUP_STORE_REMOVED,
            ));
        }
    }
    Ok(results)
}

/// Validate the needed collections are available in the MongoDB database.
pub fn validate_schema<S: BuildHasher>(
    client: &Client,
    db: &str,
    expected_collections: &HashSet<&'static str, S>,
) -> Result<Vec<ValidationResult>> {
    let collections = collections(client.db(db))?;
    let mut results = Vec::new();

    // Check all needed collections exist and are writable.
    for name in expected_collections.iter() {
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
    Ok(results)
}

/// Extra information about a collection.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct CollectionInfo {
    pub capped: bool,
    pub kind: String,
    pub read_only: bool,
}

/// Extra information about an index.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
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

/// Details of issues detected by the validation process.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct ValidationResult {
    pub collection: String,
    pub error: bool,
    pub group: &'static str,
    pub message: String,
}

impl ValidationResult {
    /// Create a `ValidationResult` for an error.
    pub fn error<S1, S2>(collection: S1, message: S2, group: &'static str) -> ValidationResult
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        ValidationResult {
            collection: collection.into(),
            error: true,
            group,
            message: message.into(),
        }
    }

    /// Create a `ValidationResult` for a non-critical issue or a suggestion.
    pub fn result<S1, S2>(collection: S1, message: S2, group: &'static str) -> ValidationResult
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        ValidationResult {
            collection: collection.into(),
            error: false,
            group,
            message: message.into(),
        }
    }
}
