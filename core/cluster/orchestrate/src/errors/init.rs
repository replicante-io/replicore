use thiserror::Error;

/// Errors during `ClusterOrchestrate` initialisation.
#[derive(Error, Debug)]
pub enum InitError {
    #[error("failed to initialise a new cluster view for {0}.{1}")]
    // (namespace_id, cluster_id)
    ClusterViewInit(String, String),

    #[error("failed to load cluster view for {0}.{1}")]
    // (namespace_id, cluster_id)
    ClusterViewLoad(String, String),

    #[error("failed to evaluate scheduling choice for {0}.{1}")]
    // (namespace_id, cluster_id)
    SchedulingChoice(String, String),
}

impl InitError {
    /// Failed to initialise cluster view.
    pub fn cluster_view_init<CID, NID>(namespace_id: NID, cluster_id: CID) -> InitError
    where
        CID: Into<String>,
        NID: Into<String>,
    {
        InitError::ClusterViewInit(namespace_id.into(), cluster_id.into())
    }

    /// Failed to load cluster view.
    pub fn cluster_view_load<CID, NID>(namespace_id: NID, cluster_id: CID) -> InitError
    where
        CID: Into<String>,
        NID: Into<String>,
    {
        InitError::ClusterViewLoad(namespace_id.into(), cluster_id.into())
    }

    /// Failed to determine scheduling choice for the cluster.
    pub fn scheduling_choice<CID, NID>(namespace_id: NID, cluster_id: CID) -> InitError
    where
        CID: Into<String>,
        NID: Into<String>,
    {
        InitError::SchedulingChoice(namespace_id.into(), cluster_id.into())
    }
}
