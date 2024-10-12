//! Pre-computed statistics about the cluster.
use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;

/// Pre-computed statistics about shards, grouped by node.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct StatsShardByNode {
    /// Number of primary shards on the node.
    pub count_primary: u64,

    /// Number of secondary shards on the node.
    pub count_secondary: u64,

    /// Number of recovering shards on the node.
    pub count_recovering: u64,

    /// Number of <role> shards on the node, for each non-standard shard role.
    pub count_others: HashMap<String, u64>,
}
