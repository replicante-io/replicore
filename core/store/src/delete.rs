//! RepliCore Control Plane persistent store operations to delete records.
use replisdk::core::models::cluster::ClusterSpec;
use replisdk::core::models::namespace::Namespace;
use replisdk::core::models::platform::Platform;

use replicore_cluster_models::ConvergeState;

use self::seal::SealDeleteOp;
use crate::ids::NamespaceID;
use crate::ids::NamespacedResourceID;
use crate::ids::NodeID;

/// Internal trait to enable delete operations on the persistent store.
pub trait DeleteOp: Into<DeleteOps> + SealDeleteOp {
    /// Type returned by the matching delete operation.
    type Response: From<DeleteResponses>;
}

/// List of all delete operations the persistent store must implement.
pub enum DeleteOps {
    /// Delete a cluster convergence state by Namespace and Name.
    ClusterConvergeState(DeleteClusterConvergeState),

    /// Delete a cluster specification by Namespace and Name.
    ClusterSpec(DeleteClusterSpec),

    /// Delete a namespace by Namespace ID.
    Namespace(DeleteNamespace),

    /// Delete a node by its ID.
    Node(NodeID),

    /// Delete a platform by Namespace and Name.
    Platform(DeletePlatform),
}

/// List of all responses from delete operations.
pub enum DeleteResponses {
    /// The operation completed successfully and does not return data.
    Success,
}

// --- High level delete operations --- //
/// Request deletion of a [`ConvergeState`] record.
pub struct DeleteClusterConvergeState(pub NamespacedResourceID);
impl From<&ConvergeState> for DeleteClusterConvergeState {
    fn from(value: &ConvergeState) -> Self {
        let value = NamespacedResourceID {
            name: value.cluster_id.clone(),
            ns_id: value.ns_id.clone(),
        };
        DeleteClusterConvergeState(value)
    }
}

/// Request deletion of a [`ClusterSpec`] record.
pub struct DeleteClusterSpec(pub NamespacedResourceID);
impl From<&ClusterSpec> for DeleteClusterSpec {
    fn from(value: &ClusterSpec) -> Self {
        let value = NamespacedResourceID {
            name: value.cluster_id.clone(),
            ns_id: value.ns_id.clone(),
        };
        DeleteClusterSpec(value)
    }
}

/// Request deletion of a [`Namespace`] record.
pub struct DeleteNamespace(pub NamespaceID);
impl From<&Namespace> for DeleteNamespace {
    fn from(value: &Namespace) -> Self {
        let id = value.id.clone();
        let value = NamespaceID { id };
        DeleteNamespace(value)
    }
}
impl From<String> for DeleteNamespace {
    fn from(value: String) -> Self {
        let value = NamespaceID { id: value };
        DeleteNamespace(value)
    }
}
impl From<&str> for DeleteNamespace {
    fn from(value: &str) -> Self {
        let id = value.to_string();
        let value = NamespaceID { id };
        DeleteNamespace(value)
    }
}

/// Request deletion of a [`Platform`] record.
pub struct DeletePlatform(pub NamespacedResourceID);
impl From<&Platform> for DeletePlatform {
    fn from(value: &Platform) -> Self {
        let value = NamespacedResourceID {
            name: value.name.clone(),
            ns_id: value.ns_id.clone(),
        };
        DeletePlatform(value)
    }
}

// --- Create internal implementation details follow --- //
/// Private module to seal implementation details.
mod seal {
    /// Super-trait to seal the [`DeleteOp`](super::DeleteOp) trait.
    pub trait SealDeleteOp {}
}

// --- Implement DeleteOp and super traits on types for transparent operations --- //
impl DeleteOp for DeleteClusterConvergeState {
    type Response = ();
}
impl SealDeleteOp for DeleteClusterConvergeState {}
impl From<DeleteClusterConvergeState> for DeleteOps {
    fn from(value: DeleteClusterConvergeState) -> Self {
        DeleteOps::ClusterConvergeState(value)
    }
}

impl DeleteOp for DeleteClusterSpec {
    type Response = ();
}
impl SealDeleteOp for DeleteClusterSpec {}
impl From<DeleteClusterSpec> for DeleteOps {
    fn from(value: DeleteClusterSpec) -> Self {
        DeleteOps::ClusterSpec(value)
    }
}
impl DeleteOp for &ClusterSpec {
    type Response = ();
}
impl SealDeleteOp for &ClusterSpec {}
impl From<&ClusterSpec> for DeleteOps {
    fn from(value: &ClusterSpec) -> Self {
        let value = DeleteClusterSpec::from(value);
        DeleteOps::ClusterSpec(value)
    }
}

impl DeleteOp for DeleteNamespace {
    type Response = ();
}
impl SealDeleteOp for DeleteNamespace {}
impl From<DeleteNamespace> for DeleteOps {
    fn from(value: DeleteNamespace) -> Self {
        DeleteOps::Namespace(value)
    }
}
impl DeleteOp for &Namespace {
    type Response = ();
}
impl SealDeleteOp for &Namespace {}
impl From<&Namespace> for DeleteOps {
    fn from(value: &Namespace) -> Self {
        let value = DeleteNamespace::from(value);
        DeleteOps::Namespace(value)
    }
}

impl DeleteOp for DeletePlatform {
    type Response = ();
}
impl SealDeleteOp for DeletePlatform {}
impl From<DeletePlatform> for DeleteOps {
    fn from(value: DeletePlatform) -> Self {
        DeleteOps::Platform(value)
    }
}
impl DeleteOp for &Platform {
    type Response = ();
}
impl SealDeleteOp for &Platform {}
impl From<&Platform> for DeleteOps {
    fn from(value: &Platform) -> Self {
        let value = DeletePlatform::from(value);
        DeleteOps::Platform(value)
    }
}

impl DeleteOp for NodeID {
    type Response = ();
}
impl SealDeleteOp for NodeID {}
impl From<NodeID> for DeleteOps {
    fn from(value: NodeID) -> Self {
        DeleteOps::Node(value)
    }
}

// --- Implement DeleteResponses conversions on return types for transparent operations --- //
impl From<DeleteResponses> for () {
    fn from(value: DeleteResponses) -> Self {
        match value {
            DeleteResponses::Success => (),
            //_ => panic!(TODO),
        }
    }
}
