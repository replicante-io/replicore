//! Data models for RepliCore Control Plane cluster related to orchestration tasks.
use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;
use time::OffsetDateTime;

/// Track state of cluster convergence cycles.
///
/// These records provide memory for the otherwise stateless convergence tasks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConvergeState {
    /// Namespace ID for the cluster this record describes.
    pub ns_id: String,

    /// Cluster ID for the cluster this record describes.
    pub cluster_id: String,

    /// Start time for converge grace periods.
    ///
    /// Many convergence operations have grace periods to avoid unsafe side effects
    /// of too frequent changes or actions.
    ///
    /// This object records the time the grace period for an operation starts.
    /// Recording the start allows configuration changes to apply as soon as they are made.
    pub graces: HashMap<String, OffsetDateTime>,
}

/// Cluster orchestration mode to use for the task.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OrchestrateMode {
    /// The cluster is deleting or deleted.
    #[serde(rename = "delete")]
    Delete,

    /// The cluster is observed but not managed.
    #[serde(rename = "observe")]
    Observe,

    /// The cluster is both observed and managed.
    #[serde(rename = "sync")]
    Sync,
}

/// User intended report of orchestration activities.
///
/// The orchestration process is a multi-phase complex task.
/// To ensure decisions made by this process can be explained a report is built
/// containing information about key data and choice rationale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrateReport {
    /// Namespace of the orchestrated cluster.
    pub ns_id: String,

    /// ID of the orchestrated cluster.
    pub cluster_id: String,

    /// Mode used to orchestrate the cluster, determined by cluster and namespace status.
    pub mode: OrchestrateMode,

    /// UTC time the orchestration task started.
    pub start_time: OffsetDateTime,
}

impl OrchestrateReport {
    /// Initialise a new orchestration task report.
    pub fn start<S1, S2>(ns_id: S1, cluster_id: S2, mode: OrchestrateMode) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            ns_id: ns_id.into(),
            cluster_id: cluster_id.into(),
            mode,
            start_time: OffsetDateTime::now_utc(),
        }
    }
}
