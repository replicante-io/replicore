//! Data models for RepliCore Control Plane cluster related to orchestration tasks.
use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value as Json;
use time::OffsetDateTime;
use uuid::Uuid;

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

impl std::fmt::Display for OrchestrateMode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Delete => write!(f, "DELETE"),
            Self::Observe => write!(f, "OBSERVE"),
            Self::Sync => write!(f, "SYNC"),
        }
    }
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

    /// Notes attachjed during cluster orchestration.
    pub notes: Vec<OrchestrateReportNote>,

    /// UTC time the orchestration task started.
    #[serde(with = "time::serde::rfc3339")]
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
            notes: Default::default(),
            start_time: OffsetDateTime::now_utc(),
        }
    }
}

/// Loosely structued object to report orchestration events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrateReportNote {
    /// Category for a note attached to orchestration reports.
    pub category: OrchestrateReportNoteCategory,

    /// Arbtrary data attached to the the note.
    pub data: HashMap<String, Json>,

    /// The note itself.
    pub message: String,
}

impl OrchestrateReportNote {
    /// Start noting an orchestration error.
    pub fn error<S>(message: S, error: anyhow::Error) -> Self
    where
        S: Into<String>,
    {
        let details = replisdk::utils::error::into_json(error);
        let mut data = HashMap::default();
        data.insert("details".into(), details);
        Self {
            category: OrchestrateReportNoteCategory::Error,
            data,
            message: message.into(),
        }
    }

    /// Attach a target Node ID to the note.
    pub fn for_node<S>(&mut self, node_id: S) -> &mut Self
    where
        S: Into<String>,
    {
        let node_id = node_id.into();
        let node_id = serde_json::Value::String(node_id);
        self.data.insert("node_id".into(), node_id);
        self
    }

    /// Attach a target Node Action ID to the note.
    pub fn for_node_action(&mut self, action_id: Uuid) -> &mut Self {
        let action_id = action_id.to_string();
        let action_id = serde_json::Value::String(action_id);
        self.data.insert("action_id".into(), action_id);
        self
    }
}

/// Category for a note attached to orchestration reports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrchestrateReportNoteCategory {
    /// Explaination of a decision made by the orchestrator.
    Decision,

    /// A non-interrupting error encountered during orchestration.
    Error,
}

impl std::fmt::Display for OrchestrateReportNoteCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Decision => write!(f, "DECISION"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}
