use thiserror::Error;

/// Errors dealing with orchestration actions.
#[derive(Error, Debug)]
pub enum ActionError {
    #[error("orchestrator action {2} with kind {3} in cluster {0}.{1} did not start when run")]
    // (namespace_id, cluster_id, action_id, kind)
    DidNotStart(String, String, String, String),

    #[error("orchestrator action {2} in cluster {0}.{1} did not finish in time and was failed")]
    // (namespace_id, cluster_id, action_id)
    TimedOut(String, String, String),

    #[error("unsupported kind {3} for orchestrator action {2} in cluster {0}.{1}")]
    // (namespace_id, cluster_id, action_id, kind)
    UnknownKind(String, String, String, String),
}

impl ActionError {
    /// The action did not start (leave the PendingSchedule state) when run.
    pub fn did_not_start<CID, NID, AID, KIND>(
        namespace_id: NID,
        cluster_id: CID,
        action_id: AID,
        kind: KIND,
    ) -> ActionError
    where
        CID: Into<String>,
        NID: Into<String>,
        AID: ToString,
        KIND: Into<String>,
    {
        ActionError::DidNotStart(
            namespace_id.into(),
            cluster_id.into(),
            action_id.to_string(),
            kind.into(),
        )
    }

    /// The action timed out before finishing.
    pub fn timed_out<CID, NID, AID>(
        namespace_id: NID,
        cluster_id: CID,
        action_id: AID,
    ) -> ActionError
    where
        CID: Into<String>,
        NID: Into<String>,
        AID: ToString,
    {
        ActionError::TimedOut(
            namespace_id.into(),
            cluster_id.into(),
            action_id.to_string(),
        )
    }

    /// Unable to find the requested action kind.
    pub fn unknown_kind<CID, NID, AID, KIND>(
        namespace_id: NID,
        cluster_id: CID,
        action_id: AID,
        kind: KIND,
    ) -> ActionError
    where
        CID: Into<String>,
        NID: Into<String>,
        AID: ToString,
        KIND: Into<String>,
    {
        ActionError::UnknownKind(
            namespace_id.into(),
            cluster_id.into(),
            action_id.to_string(),
            kind.into(),
        )
    }
}
