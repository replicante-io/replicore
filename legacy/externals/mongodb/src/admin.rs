use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::BuildHasher;

use failure::ResultExt;
use mongodb::results::CollectionType;
use mongodb::sync::Client;
use mongodb::sync::Database;

use crate::metrics::MONGODB_OPS_COUNT;
use crate::metrics::MONGODB_OPS_DURATION;
use crate::metrics::MONGODB_OP_ERRORS_COUNT;
use crate::ErrorKind;
use crate::Result;

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
        .list_collections(None, None)
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
        let name = collection.name;
        let kind = match collection.collection_type {
            CollectionType::Collection => "collection",
            CollectionType::View => "view",
            _ => "<unknown type>",
        }
        .into();
        let capped = collection.options.capped.unwrap_or(false);
        let read_only = collection.info.read_only;
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

/// Validate the list of deprecated collections agains the MongoDB database.
pub fn validate_removed_collections<S: BuildHasher>(
    client: &Client,
    db: &str,
    removed_collections: &HashSet<&'static str, S>,
) -> Result<Vec<ValidationResult>> {
    let collections = collections(client.database(db))?;
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
    let collections = collections(client.database(db))?;
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
