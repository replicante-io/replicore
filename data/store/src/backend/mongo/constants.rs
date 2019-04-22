use std::collections::HashMap;
use std::collections::HashSet;

use bson::ordered::OrderedDocument;

use super::validate::IndexInfo;

pub const COLLECTION_AGENTS: &str = "agents";
pub const COLLECTION_AGENTS_INFO: &str = "agents_info";
pub const COLLECTION_CLUSTER_META: &str = "clusters_meta";
pub const COLLECTION_DISCOVERIES: &str = "discoveries";
pub const COLLECTION_EVENTS: &str = "events";
pub const COLLECTION_NODES: &str = "nodes";
pub const COLLECTION_SHARDS: &str = "shards";

pub const TOP_CLUSTERS_LIMIT: u32 = 10;

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
        set.insert(COLLECTION_AGENTS);
        set.insert(COLLECTION_AGENTS_INFO);
        set.insert(COLLECTION_CLUSTER_META);
        set.insert(COLLECTION_DISCOVERIES);
        set.insert(COLLECTION_EVENTS);
        set.insert(COLLECTION_NODES);
        set.insert(COLLECTION_SHARDS);
        set
    };
    pub static ref VALIDATE_INDEXES_NEEDED: HashMap<&'static str, Vec<IndexInfo>> = {
        let mut map = HashMap::new();
        map.insert(
            COLLECTION_AGENTS,
            vec![IndexInfo {
                expires: false,
                key: vec![("cluster_id".into(), 1), ("host".into(), 1)],
                unique: true,
            }],
        );
        map.insert(
            COLLECTION_AGENTS_INFO,
            vec![IndexInfo {
                expires: false,
                key: vec![("cluster_id".into(), 1), ("host".into(), 1)],
                unique: true,
            }],
        );
        map.insert(
            COLLECTION_CLUSTER_META,
            vec![IndexInfo {
                expires: false,
                key: vec![("cluster_id".into(), 1)],
                unique: true,
            }],
        );
        map.insert(
            COLLECTION_DISCOVERIES,
            vec![IndexInfo {
                expires: false,
                key: vec![("cluster_id".into(), 1)],
                unique: true,
            }],
        );
        map.insert(COLLECTION_EVENTS, vec![]);
        map.insert(
            COLLECTION_NODES,
            vec![IndexInfo {
                expires: false,
                key: vec![("cluster_id".into(), 1), ("node_id".into(), 1)],
                unique: true,
            }],
        );
        map.insert(
            COLLECTION_SHARDS,
            vec![IndexInfo {
                expires: false,
                key: vec![
                    ("cluster_id".into(), 1),
                    ("node_id".into(), 1),
                    ("shard_id".into(), 1),
                ],
                unique: true,
            }],
        );
        map
    };
    pub static ref VALIDATE_INDEXES_SUGGESTED: HashMap<&'static str, Vec<IndexInfo>> = {
        let mut map = HashMap::new();
        map.insert(COLLECTION_AGENTS, vec![]);
        map.insert(COLLECTION_AGENTS_INFO, vec![]);
        map.insert(
            COLLECTION_CLUSTER_META,
            vec![
                IndexInfo {
                    expires: false,
                    key: vec![
                        ("shards".into(), -1),
                        ("nodes".into(), -1),
                        ("cluster_id".into(), 1),
                    ],
                    unique: false,
                },
                IndexInfo {
                    expires: false,
                    key: vec![("cluster_display_name".into(), 1)],
                    unique: false,
                },
            ],
        );
        map.insert(COLLECTION_DISCOVERIES, vec![]);
        map.insert(COLLECTION_EVENTS, vec![]);
        map.insert(COLLECTION_NODES, vec![]);
        map.insert(COLLECTION_SHARDS, vec![]);
        map
    };
}
