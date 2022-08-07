use thiserror::Error;

/// Action or node related errors during cluster orchestration.
///
/// These are errors in the execution of actions or interactions between Core and nodes.
/// In other words these are errors that should NOT prevent processing other stages/actions/nodes.
#[derive(Error, Debug)]
pub enum SyncError {
    #[error("failed to connect to node {2} in cluster {0}.{1}")]
    // (namespace_id, cluster_id, node_id)
    ClientConnect(String, String, String),

    #[error("invalid {3} response from node {2} in cluster {0}.{1}")]
    // (namespace_id, cluster_id, node_id, api_action)
    ClientResponse(String, String, String, String),

    #[error("failed to incrementally update cluster view for cluster {0}.{1} with data from {2}")]
    // (namespace_id, cluster_id, node_id)
    // NOTE: Should this be a full sync ending error?
    //       Does a possibly corrupt view warrant never orchestrating a cluster?
    ClusterViewUpdate(String, String, String),

    #[error("failed to emit {2} event about cluster {0}.{1}")]
    // (namespace_id, cluster_id, event_code)
    // NOTE: Should this be a full sync ending error?
    //       Does a core dependency error warrant never orchestrating a cluster?
    //       What if the issue is actually the client data instead of the streaming service?
    EventEmit(String, String, String),

    #[error("failed to emit {3} event about node {2} for cluster {0}.{1}")]
    // (namespace_id, cluster_id, node_id, event_code)
    // NOTE: Should this be a full sync ending error?
    //       Does a core dependency error warrant never orchestrating a cluster?
    //       What if the issue is actually the client data instead of the streaming service?
    EventEmitForNode(String, String, String, String),

    #[error("unable to find action with ID {3} to schedule on node {2} for cluster {0}.{1}")]
    // (namespace_id, cluster_id, node_id, action_id)
    ExpectedActionNotFound(String, String, String, String),

    #[error("failed to persist record about cluster {0}.{1}")]
    // (namespace_id, cluster_id)
    // NOTE: Should this be a full sync ending error?
    //       Does a core dependency error warrant never orchestrating a cluster?
    //       What if the issue is actually the client data instead of the DB service?
    StorePersist(String, String),

    #[error("failed to persist record about node {2} for cluster {0}.{1}")]
    // (namespace_id, cluster_id, node_id)
    // NOTE: Should this be a full sync ending error?
    //       Does a core dependency error warrant never orchestrating a cluster?
    //       What if the issue is actually the client data instead of the DB service?
    StorePersistForNode(String, String, String),

    #[error("failed to read record about node {2} for cluster {0}.{1}")]
    // (namespace_id, cluster_id, node_id)
    // NOTE: Should this be a full sync ending error?
    //       Does a core dependency error warrant never orchestrating a cluster?
    //       What if the issue is actually the client data instead of the DB service?
    StoreRead(String, String, String),
}

impl SyncError {
    /// Errors establishing a connection to a specific node.
    pub fn client_connect<CID, NID, NODE>(
        namespace_id: NID,
        cluster_id: CID,
        node_id: NODE,
    ) -> SyncError
    where
        CID: Into<String>,
        NID: Into<String>,
        NODE: Into<String>,
    {
        SyncError::ClientConnect(namespace_id.into(), cluster_id.into(), node_id.into())
    }

    /// Errors response received from a node.
    pub fn client_response<CID, NID, NODE, API>(
        namespace_id: NID,
        cluster_id: CID,
        node_id: NODE,
        api_action: API,
    ) -> SyncError
    where
        CID: Into<String>,
        NID: Into<String>,
        NODE: Into<String>,
        API: Into<String>,
    {
        SyncError::ClientResponse(
            namespace_id.into(),
            cluster_id.into(),
            node_id.into(),
            api_action.into(),
        )
    }

    /// Errors updating the incremental cluster view with node information.
    pub fn cluster_view_update<CID, NID, NODE>(
        namespace_id: NID,
        cluster_id: CID,
        node_id: NODE,
    ) -> SyncError
    where
        CID: Into<String>,
        NID: Into<String>,
        NODE: Into<String>,
    {
        SyncError::ClusterViewUpdate(namespace_id.into(), cluster_id.into(), node_id.into())
    }

    // Errors emitting sync events to the streaming platform.
    pub fn event_emit<CID, NID, CODE>(
        namespace_id: NID,
        cluster_id: CID,
        event_code: CODE,
    ) -> SyncError
    where
        CID: Into<String>,
        NID: Into<String>,
        CODE: Into<String>,
    {
        SyncError::EventEmit(namespace_id.into(), cluster_id.into(), event_code.into())
    }

    /// Errors emitting sync events related to nodes to the streaming platform.
    pub fn event_emit_for_node<CID, NID, NODE, CODE>(
        namespace_id: NID,
        cluster_id: CID,
        node_id: NODE,
        event_code: CODE,
    ) -> SyncError
    where
        CID: Into<String>,
        NID: Into<String>,
        NODE: Into<String>,
        CODE: Into<String>,
    {
        SyncError::EventEmitForNode(
            namespace_id.into(),
            cluster_id.into(),
            node_id.into(),
            event_code.into(),
        )
    }

    pub fn expected_action_not_found<CID, NID, NODE, AID>(
        namespace_id: NID,
        cluster_id: CID,
        node_id: NODE,
        action_id: AID,
    ) -> SyncError
    where
        CID: Into<String>,
        NID: Into<String>,
        NODE: Into<String>,
        AID: ToString,
    {
        SyncError::ExpectedActionNotFound(
            namespace_id.into(),
            cluster_id.into(),
            node_id.into(),
            action_id.to_string(),
        )
    }

    /// Errors persisting cluster records to the store.
    pub fn store_persist<CID, NID>(namespace_id: NID, cluster_id: CID) -> SyncError
    where
        CID: Into<String>,
        NID: Into<String>,
    {
        SyncError::StorePersist(namespace_id.into(), cluster_id.into())
    }

    /// Errors persisting node records to the store.
    pub fn store_persist_for_node<CID, NID, NODE>(
        namespace_id: NID,
        cluster_id: CID,
        node_id: NODE,
    ) -> SyncError
    where
        CID: Into<String>,
        NID: Into<String>,
        NODE: Into<String>,
    {
        SyncError::StorePersistForNode(namespace_id.into(), cluster_id.into(), node_id.into())
    }

    /// Error reading records from the store.
    pub fn store_read<CID, NID, NODE>(
        namespace_id: NID,
        cluster_id: CID,
        node_id: NODE,
    ) -> SyncError
    where
        CID: Into<String>,
        NID: Into<String>,
        NODE: Into<String>,
    {
        SyncError::StoreRead(namespace_id.into(), cluster_id.into(), node_id.into())
    }
}
