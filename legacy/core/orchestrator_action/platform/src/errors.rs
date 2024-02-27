use thiserror::Error;

/// Errors relating to actions arguments.
#[derive(Debug, Error)]
pub enum Arguments {
    #[error("action for cluster {0} trying to create node for cluster {1}")]
    // (action_cluster_id, arguments_cluster_id)
    InvalidClusterScope(String, String),
}

impl Arguments {
    /// An action's arguments are referencing a cluster other then the action's cluster.
    pub fn invalid_cluster_scope<S1, S2>(action_cluster_id: S1, arguments_cluster_id: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Arguments::InvalidClusterScope(action_cluster_id.into(), arguments_cluster_id.into())
    }
}

/// Errors relating to or interacting with a Platform instance.
#[derive(Debug, Error)]
pub enum Platform {
    #[error("the referenced platform is not active")]
    NotActive,

    #[error("unable to find the referenced platform")]
    NotFound,
}
