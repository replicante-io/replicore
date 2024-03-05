//! User intended report of orchestration activities.
use serde::Deserialize;
use serde::Serialize;
use time::OffsetDateTime;

use crate::init::OrchestrateMode;

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
