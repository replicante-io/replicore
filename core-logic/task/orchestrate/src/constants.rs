//! Constants used during Orchestrate operations.

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
