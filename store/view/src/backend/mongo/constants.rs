use std::collections::HashMap;
use std::collections::HashSet;

use bson::bson;
use bson::doc;
use bson::ordered::OrderedDocument;
use lazy_static::lazy_static;

use replicante_externals_mongodb::admin::IndexInfo;

pub const COLLECTION_ACTIONS: &str = "actions";
pub const COLLECTION_ACTIONS_HISTORY: &str = "actions_history";
pub const COLLECTION_EVENTS: &str = "events";
pub const MAX_ACTIONS_SEARCH: i64 = 100;

lazy_static! {
    pub static ref EVENTS_FILTER_NOT_SNAPSHOT: OrderedDocument = doc! {"$nin" => [
        "SNAPSHOT_AGENT",
        "SNAPSHOT_AGENT_INFO",
        "SNAPSHOT_DISCOVERY",
        "SNAPSHOT_NODE",
        "SNAPSHOT_SHARD",
    ]};
    pub static ref VALIDATE_EXPECTED_COLLECTIONS: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.insert(COLLECTION_ACTIONS);
        set.insert(COLLECTION_ACTIONS_HISTORY);
        set.insert(COLLECTION_EVENTS);
        set
    };
    pub static ref VALIDATE_INDEXES_NEEDED: HashMap<&'static str, Vec<IndexInfo>> = {
        let mut map = HashMap::new();
        map.insert(
            COLLECTION_ACTIONS,
            vec![
                IndexInfo {
                    expires: false,
                    key: vec![("cluster_id".into(), 1), ("action_id".into(), 1)],
                    unique: true,
                },
                IndexInfo {
                    expires: true,
                    key: vec![("finished_ts".into(), 1)],
                    unique: false,
                },
            ],
        );
        map.insert(
            COLLECTION_ACTIONS_HISTORY,
            vec![
                IndexInfo {
                    expires: false,
                    key: vec![
                        ("cluster_id".into(), 1),
                        ("action_id".into(), 1),
                        ("timestamp".into(), 1),
                        ("state".into(), 1),
                    ],
                    unique: true,
                },
                IndexInfo {
                    expires: true,
                    key: vec![("finished_ts".into(), 1)],
                    unique: false,
                },
            ],
        );
        map.insert(
            COLLECTION_EVENTS,
            vec![IndexInfo {
                expires: true,
                key: vec![("timestamp".into(), 1)],
                unique: false,
            }],
        );
        map
    };
    pub static ref VALIDATE_INDEXES_SUGGESTED: HashMap<&'static str, Vec<IndexInfo>> = {
        let mut map = HashMap::new();
        map.insert(
            COLLECTION_ACTIONS,
            vec![IndexInfo {
                expires: false,
                key: vec![("cluster_id".into(), 1), ("created_ts".into(), -1)],
                unique: false,
            }],
        );
        map.insert(COLLECTION_ACTIONS_HISTORY, vec![]);
        map.insert(COLLECTION_EVENTS, vec![]);
        map
    };
}
