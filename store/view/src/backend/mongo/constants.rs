use std::collections::HashSet;

use bson::doc;
use lazy_static::lazy_static;

pub const COLLECTION_ACTIONS_HISTORY: &str = "actions_history";
pub const COLLECTION_ACTIONS: &str = "actions";
pub const COLLECTION_CLUSTER_ORCHESTRATE_REPORT: &str = "cluster_orchestrate_report";
pub const COLLECTION_EVENTS: &str = "events";
pub const MAX_ACTIONS_SEARCH: i64 = 100;

lazy_static! {
    pub static ref VALIDATE_EXPECTED_COLLECTIONS: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.insert(COLLECTION_ACTIONS);
        set.insert(COLLECTION_ACTIONS_HISTORY);
        set.insert(COLLECTION_CLUSTER_ORCHESTRATE_REPORT);
        set.insert(COLLECTION_EVENTS);
        set
    };
}
