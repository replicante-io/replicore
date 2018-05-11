use std::collections::HashMap;
use std::collections::HashSet;

use bson;
use bson::Bson;
use bson::Document;
use error_chain::ChainedError;

use mongodb::Client;
use mongodb::ThreadedClient;
use mongodb::coll::options::FindOptions;
use mongodb::db::ThreadedDatabase;

use replicante_data_models::Agent;
//use replicante_data_models::AgentInfo;

use super::super::super::Cursor;
use super::super::super::Error;
use super::super::super::ErrorKind;

use super::Result;
use super::ResultExt;
use super::ValidationResult;

use super::constants::COLLECTION_AGENTS;
use super::constants::COLLECTION_AGENTS_INFO;
use super::constants::COLLECTION_CLUSTER_META;
use super::constants::COLLECTION_DISCOVERIES;
use super::constants::COLLECTION_EVENTS;
use super::constants::COLLECTION_NODES;
use super::constants::COLLECTION_SHARDS;
use super::constants::EXPECTED_COLLECTIONS;

use super::metrics::MONGODB_OPS_COUNT;
use super::metrics::MONGODB_OPS_DURATION;
use super::metrics::MONGODB_OP_ERRORS_COUNT;


const GROUP_PERF_INDEX: &'static str = "perf/index";
const GROUP_STORE_ERROR: &'static str = "store/error";
const GROUP_STORE_MISSING: &'static str = "store/missing";
const GROUP_STORE_SCHEMA: &'static str = "store/schema";


lazy_static! {
    static ref INDEX_NEEDED: HashMap<&'static str, Vec<IndexInfo>> = {
        let mut map = HashMap::new();

        map.insert(COLLECTION_AGENTS, vec![IndexInfo {
            expires: false,
            key: vec![("cluster".into(), 1), ("host".into(), 1)],
            unique: true
        }]);
        map.insert(COLLECTION_AGENTS_INFO, vec![IndexInfo {
            expires: false,
            key: vec![("cluster".into(), 1), ("host".into(), 1)],
            unique: true
        }]);
        map.insert(COLLECTION_CLUSTER_META, vec![IndexInfo {
            expires: false,
            key: vec![("name".into(), 1)],
            unique: true
        }]);
        map.insert(COLLECTION_DISCOVERIES, vec![IndexInfo {
            expires: false,
            key: vec![("name".into(), 1)],
            unique: true
        }]);
        map.insert(COLLECTION_EVENTS, vec![]);
        map.insert(COLLECTION_NODES, vec![IndexInfo {
            expires: false,
            key: vec![("cluster".into(), 1), ("name".into(), 1)],
            unique: true
        }]);
        map.insert(COLLECTION_SHARDS, vec![IndexInfo {
            expires: false,
            key: vec![("cluster".into(), 1), ("node".into(), 1), ("id".into(), 1)],
            unique: true
        }]);

        map
    };

    static ref INDEX_SUGGESTED: HashMap<&'static str, Vec<IndexInfo>> = {
        let mut map = HashMap::new();

        map.insert(COLLECTION_AGENTS, vec![]);
        map.insert(COLLECTION_AGENTS_INFO, vec![]);
        map.insert(COLLECTION_CLUSTER_META, vec![IndexInfo {
            expires: false,
            key: vec![("nodes".into(), -1), ("name".into(), 1)],
            unique: false
        }]);
        map.insert(COLLECTION_DISCOVERIES, vec![]);
        map.insert(COLLECTION_EVENTS, vec![]);
        map.insert(COLLECTION_NODES, vec![]);
        map.insert(COLLECTION_SHARDS, vec![]);

        map
    };
}


/// Extra information about a collection.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
struct CollectionInfo {
    pub capped: bool,
    pub kind: String,
    pub read_only: bool,
}


/// Subset of the validator looking at the dataset.
pub struct DataValidator {
    client: Client,
    db: String,
}

impl DataValidator {
    pub fn new(db: String, client: Client) -> DataValidator {
        DataValidator { client, db }
    }

    /// Iterate over the agents in the store.
    pub fn agents(&self) -> Result<Cursor<Agent>> {
        let sort = doc! { "_id" => 1 };
        let mut options = FindOptions::new();
        options.sort = Some(sort);
        // TODO: metrics
        let db = self.client.db(&self.db);
        let collection = db.collection(COLLECTION_AGENTS);
        let cursor = collection.find(None, Some(options))?;
        let cursor = cursor.map(|item| match item {
            Err(error) => Err(error.into()),
            Ok(item) => {
                let id = item.get_object_id("_id")
                    .map(|id| id.to_hex())
                    .unwrap_or("<NO ID>".into());
                match bson::from_bson::<Agent>(bson::Bson::Document(item)) {
                    Ok(item) => Ok(item),
                    Err(error) => {
                        let error: Error = error.into();
                        let error = error.display_chain().to_string();
                        Err(ErrorKind::UnableToParseModel(id, error).into())
                    }
                }
            }
        });
        let cursor = Cursor(Box::new(cursor));
        Ok(cursor)
    }

    /// Count the agents in the store.
    pub fn agents_count(&self) -> Result<u64> {
        // TODO: metrics
        let db = self.client.db(&self.db);
        let collection = db.collection(COLLECTION_AGENTS);
        collection.count(None, None).map(|count| count as u64)
            .chain_err(|| "Failed to count agents")
    }
}


/// Extra information about an index.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
struct IndexInfo {
    pub expires: bool,
    pub key: Vec<(String, i8)>,
    pub unique: bool,
}

impl IndexInfo {
    pub fn parse(document: Document) -> Result<IndexInfo> {
        let mut keys = Vec::new();
        for (key, direction) in document.get_document("key").expect("index has no key").iter() {
            let direction = match direction {
                &Bson::FloatingPoint(f) if f == 1.0 => 1,
                &Bson::FloatingPoint(f) if f == -1.0 => -1,
                &Bson::I32(i) if i == 1 => 1,
                &Bson::I32(i) if i == -1 => -1,
                &Bson::I64(i) if i == 1 => 1,
                &Bson::I64(i) if i == -1 => -1,
                _ => panic!("key direction is not 1 or -1")
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


/// Subset of the validator looking for indexes configuration.
pub struct IndexValidator {
    client: Client,
    db: String,
}

impl IndexValidator {
    pub fn new(db: String, client: Client) -> IndexValidator {
        IndexValidator { client, db }
    }

    /// Check for the existence of needed and suggested indexes.
    pub fn indexes(&self) -> Result<Vec<ValidationResult>> {
        let mut results = Vec::new();
        for collection in EXPECTED_COLLECTIONS.iter() {
            let indexes = self.indexes_set(collection)?;

            let needed = INDEX_NEEDED.get(collection)
                .expect(format!("needed indexes not configured for '{}'", collection).as_ref());
            for index in needed {
                if !indexes.contains(index) {
                    results.push(ValidationResult::error(
                        collection.clone(),
                        format!("missing required index: {:?}", index),
                        GROUP_STORE_MISSING
                    ));
                }
            }

            let suggested = INDEX_SUGGESTED.get(collection)
                .expect(format!("suggested indexes not configured for '{}'", collection).as_ref());
            for index in suggested {
                if !indexes.contains(index) {
                    results.push(ValidationResult::result(
                        collection.clone(),
                        format!("recommended index not found: {:?}", index),
                        GROUP_PERF_INDEX
                    ));
                }
            }
        }
        Ok(results)
    }
}

impl IndexValidator {
    fn indexes_set(&self, collection: &'static str) -> Result<HashSet<IndexInfo>> {
        let db = self.client.db(&self.db);
        let collection = db.collection(collection);
        let mut indexes = HashSet::new();
        MONGODB_OPS_COUNT.with_label_values(&["listIndexes"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["listIndexes"]).start_timer();
        let cursor = collection.list_indexes().map_err(|error| {
            MONGODB_OP_ERRORS_COUNT.with_label_values(&["listIndexes"]).inc();
            error
        })?;
        for index in cursor {
            let index = IndexInfo::parse(index?)?;
            indexes.insert(index);
        }
        Ok(indexes)
    }
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
