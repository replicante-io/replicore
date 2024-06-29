//! RepliCore Control Plane persistent store operations to persist records.
use replisdk::core::models::cluster::ClusterDiscovery;
use replisdk::core::models::cluster::ClusterSpec;
use replisdk::core::models::naction::NAction;
use replisdk::core::models::namespace::Namespace;
use replisdk::core::models::node::Node;
use replisdk::core::models::node::Shard;
use replisdk::core::models::node::StoreExtras;
use replisdk::core::models::oaction::OAction;
use replisdk::core::models::platform::Platform;

use replicore_cluster_models::ConvergeState;

use self::seal::SealPersistOp;

/// Internal trait to enable persist operations on the persistent store.
pub trait PersistOp: Into<PersistOps> + SealPersistOp {
    /// Type returned by the matching persist operation.
    type Response: From<PersistResponses>;
}

/// List of all persist operations the persistent store must implement.
pub enum PersistOps {
    /// Persist a cluster convergence state.
    ClusterConvergeState(ConvergeState),

    /// Persist a cluster discovery record.
    ClusterDiscovery(ClusterDiscovery),

    /// Persist a cluster specification record.
    ClusterSpec(ClusterSpec),

    /// Persist a node action record.
    NAction(NAction),

    /// Persist a namespace record.
    Namespace(Namespace),

    /// Persist a cluster node record.
    Node(Node),

    /// Persist an orchestrator action record.
    OAction(OAction),

    /// Persist a platform record.
    Platform(Platform),

    /// Persist a cluster node's Shard record.
    Shard(Shard),

    /// Persist a cluster node's StoreExtras record.
    StoreExtras(StoreExtras),
}

/// List of all responses from persist operations.
pub enum PersistResponses {
    /// The operation completed successfully and does not return data.
    Success,
}

// --- High level query operations --- //
// TODO: define as needed or remove if none after feature parity.

// --- Create internal implementation details follow --- //
/// Private module to seal implementation details.
mod seal {
    /// Super-trait to seal the [`PersistOp`](super::PersistOp) trait.
    pub trait SealPersistOp {}
}

// --- Implement PersistOp and super traits on types for transparent operations --- //
impl PersistOp for ConvergeState {
    type Response = ();
}
impl SealPersistOp for ConvergeState {}
impl From<ConvergeState> for PersistOps {
    fn from(value: ConvergeState) -> Self {
        PersistOps::ClusterConvergeState(value)
    }
}

impl PersistOp for ClusterDiscovery {
    type Response = ();
}
impl SealPersistOp for ClusterDiscovery {}
impl From<ClusterDiscovery> for PersistOps {
    fn from(value: ClusterDiscovery) -> Self {
        PersistOps::ClusterDiscovery(value)
    }
}

impl PersistOp for ClusterSpec {
    type Response = ();
}
impl SealPersistOp for ClusterSpec {}
impl From<ClusterSpec> for PersistOps {
    fn from(value: ClusterSpec) -> Self {
        PersistOps::ClusterSpec(value)
    }
}

impl PersistOp for NAction {
    type Response = ();
}
impl SealPersistOp for NAction {}
impl From<NAction> for PersistOps {
    fn from(value: NAction) -> Self {
        PersistOps::NAction(value)
    }
}

impl PersistOp for Namespace {
    type Response = ();
}
impl SealPersistOp for Namespace {}
impl From<Namespace> for PersistOps {
    fn from(value: Namespace) -> Self {
        PersistOps::Namespace(value)
    }
}

impl PersistOp for Node {
    type Response = ();
}
impl SealPersistOp for Node {}
impl From<Node> for PersistOps {
    fn from(value: Node) -> Self {
        PersistOps::Node(value)
    }
}

impl PersistOp for OAction {
    type Response = ();
}
impl SealPersistOp for OAction {}
impl From<OAction> for PersistOps {
    fn from(value: OAction) -> Self {
        PersistOps::OAction(value)
    }
}

impl PersistOp for Platform {
    type Response = ();
}
impl SealPersistOp for Platform {}
impl From<Platform> for PersistOps {
    fn from(value: Platform) -> Self {
        PersistOps::Platform(value)
    }
}

impl PersistOp for Shard {
    type Response = ();
}
impl SealPersistOp for Shard {}
impl From<Shard> for PersistOps {
    fn from(value: Shard) -> Self {
        PersistOps::Shard(value)
    }
}

impl PersistOp for StoreExtras {
    type Response = ();
}
impl SealPersistOp for StoreExtras {}
impl From<StoreExtras> for PersistOps {
    fn from(value: StoreExtras) -> Self {
        PersistOps::StoreExtras(value)
    }
}

// --- Implement PersistResponses conversions on return types for transparent operations --- //
impl From<PersistResponses> for () {
    fn from(value: PersistResponses) -> Self {
        match value {
            PersistResponses::Success => (),
            //_ => panic!(TODO),
        }
    }
}
