//! Constants used during Orchestrate operations.

/// Event code emitted when a new node action was first seen during sync.
pub const NACTION_SYNC_NEW: &str = "NACTION_SYNC_NEW";

/// Event code emitted when a node action was updated during sync.
pub const NACTION_SYNC_UPDATE: &str = "NACTION_SYNC_UPDATE";

/// Event code emitted when a new node was first seen during sync.
pub const NODE_SYNC_NEW: &str = "NODE_SYNC_NEW";

/// Event code emitted when a node was updated during sync.
pub const NODE_SYNC_UPDATE: &str = "NODE_SYNC_UPDATE";

/// Event code emitted when an orchestrator action has been cancelled.
pub const OACTION_CANCEL: &str = "OACTION_CANCEL";

/// Event code emitted when an orchestrator action has failed.
pub const OACTION_FAIL: &str = "OACTION_FAIL";

/// Event code emitted when an orchestrator action has succeeded.
pub const OACTION_SUCCESS: &str = "OACTION_SUCCESS";

/// Event code emitted when an orchestrator action is updated.
pub const OACTION_UPDATE: &str = "OACTION_UPDATE";

/// Event code emitted when the orchestration cycle is complete and a report emitted.
pub const ORCHESTRATE_REPORT: &str = "ORCHESTRATE_REPORT";

/// Event code emitted when a new shard was first seen on a node during sync.
pub const SHARD_SYNC_NEW: &str = "SHARD_SYNC_NEW";

/// Event code emitted when a shard was updated on a node during sync.
pub const SHARD_SYNC_UPDATE: &str = "SHARD_SYNC_UPDATE";

/// Event code emitted when store extras for a node were first seen during sync.
pub const STORE_EXTRAS_SYNC_NEW: &str = "STORE_EXTRAS_SYNC_NEW";

/// Event code emitted when store extras for a node were updated during sync.
pub const STORE_EXTRAS_SYNC_UPDATE: &str = "STORE_EXTRAS_SYNC_UPDATE";
