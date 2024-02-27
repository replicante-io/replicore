use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

mod builder;
mod error;

use super::sched_choice::SchedChoice;

#[cfg(test)]
mod tests;

pub use self::builder::OrchestrateReportBuilder;
pub use self::error::OrchestrateReportError;

/// Report debugging information about a cluster orchestration action.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct OrchestrateReport {
    // Cluster identification attributes.
    pub namespace: String,
    pub cluster_id: String,

    // Orchestration task metadata.
    pub start_time: DateTime<Utc>,
    pub duration: Duration,
    /// Outcome of the orchestrate operation.
    pub outcome: OrchestrateReportOutcome,

    // Orchestration task details.
    /// Details of action scheduling choices made.
    pub action_scheduling_choices: Option<SchedChoice>,
    /// Number of node actions incomplete in core but no longer reported.
    pub node_actions_lost: u64,
    /// Number of node actions scheduling attempts that failed.
    pub node_actions_schedule_failed: u64,
    /// Number of node actions scheduled on any node.
    pub node_actions_scheduled: u64,
    /// Number of cluster nodes for which sync failed.
    pub nodes_failed: u64,
    /// Number of cluster nodes for which sync was attempted.
    pub nodes_synced: u64,
}

/// Details about the outcome of an orchestrate operation.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct OrchestrateReportOutcome {
    /// Error that caused the operation to fail (if success is false).
    pub error: Option<String>,

    /// All available error causes that led the operation to fail (if success is false).
    pub error_causes: Vec<String>,

    /// Indicate if the operation was successful or not.
    pub success: bool,
}

impl From<&anyhow::Result<()>> for OrchestrateReportOutcome {
    fn from(result: &anyhow::Result<()>) -> OrchestrateReportOutcome {
        if let Err(error) = result {
            OrchestrateReportOutcome {
                error: Some(error.to_string()),
                error_causes: error.chain().skip(1).map(ToString::to_string).collect(),
                success: false,
            }
        } else {
            OrchestrateReportOutcome {
                error: None,
                error_causes: Vec::new(),
                success: true,
            }
        }
    }
}
