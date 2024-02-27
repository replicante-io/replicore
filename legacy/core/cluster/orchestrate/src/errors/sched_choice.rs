use thiserror::Error;

/// Errors encountered while computing a scheduling choice.
#[derive(Error, Debug)]
pub enum SchedChoiceError {
    /// An orchestrator action in the cluster is not available in the registry.
    ///
    /// Attached to this error are:
    ///
    ///   * The missing orchestrator action kind.
    ///   * The cluster namespace.
    ///   * The cluster id.
    #[error("unknown orchestrator action {0} in cluster {1}.{2}")]
    OrchestratorActionNotFound(String, String, String),
}

impl SchedChoiceError {
    /// Return a `SchedChoiceError::OrchestratorActionNotFound` error.
    pub fn orchestrator_action_not_found<KIND, NS, CID>(
        kind: KIND,
        namespace: NS,
        cluster: CID,
    ) -> Self
    where
        KIND: Into<String>,
        NS: Into<String>,
        CID: Into<String>,
    {
        let kind = kind.into();
        let namespace = namespace.into();
        let cluster = cluster.into();
        SchedChoiceError::OrchestratorActionNotFound(kind, namespace, cluster)
    }
}
