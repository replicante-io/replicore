//! RepliCore Control Plane persistent store operations to query records.
use anyhow::Result;
use futures::Stream;
use uuid::Uuid;

use replisdk::core::models::api::ClusterSpecEntry;
use replisdk::core::models::api::NActionEntry;
use replisdk::core::models::api::NamespaceEntry;
use replisdk::core::models::api::OActionEntry;
use replisdk::core::models::api::PlatformEntry;
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

use self::seal::SealQueryOp;
use crate::ids::NamespaceID;
use crate::ids::NamespacedResourceID;
use crate::ids::OActionID;

/// Internal trait to enable query operations on the persistent store.
pub trait QueryOp: Into<QueryOps> + SealQueryOp {
    /// Type returned by the matching query operation.
    type Response: From<QueryResponses>;
}

/// List of all query operations the persistent store must implement.
pub enum QueryOps {
    /// Query a cluster convergence state record by Namespace and Cluster ID.
    ClusterConvergeState(NamespacedResourceID),

    /// Query a cluster discovery record by Namespace and Cluster ID.
    ClusterDiscovery(NamespacedResourceID),

    /// Query a cluster specification by Namespace ID and Resource Name.
    ClusterSpec(NamespacedResourceID),

    /// List the summary information of all cluster specs in a namespace, sorted alphabetically.
    ListClusterSpecs(NamespaceID),

    /// List all node actions for a specific cluster.
    ListNActions(ListNActions),

    /// List summary information of all known namespaces, sorted alphabetically.
    ListNamespaces,

    /// List nodes for a cluster.
    ListNodes(NamespacedResourceID),

    /// List all orchestrator actions for a specific cluster.
    ListOActions(ListOActions),

    /// List summary information about known platforms in the namespace, sorted alphabetically.
    ListPlatforms(NamespaceID),

    /// List shards in a cluster.
    ListShards(ListShards),

    /// List store extras for all nodes in a cluster.
    ListStoreExtras(NamespacedResourceID),

    /// Query a namespace by Namespace ID.
    Namespace(LookupNamespace),

    /// Query an orchestrator action by namespace, cluster and action ID.
    OAction(LookupOAction),

    /// Query a platform by Namespace ID and Resource Name.
    Platform(NamespacedResourceID),

    /// Iterate over all unfinished node actions for a cluster.
    UnfinishedNAction(NamespacedResourceID),

    /// Iterate over all unfinished orchestrator actions for a cluster.
    UnfinishedOAction(NamespacedResourceID),
}

/// List of all responses from query operations.
pub enum QueryResponses {
    /// Return a [`ConvergeState`], if one was found matching the query.
    ClusterConvergeState(Option<ConvergeState>),

    /// Return a [`ClusterDiscovery`], if one was found matching the query.
    ClusterDiscovery(Option<ClusterDiscovery>),

    /// Return a [`ClusterSpec`], if one was found matching the query.
    ClusterSpec(Option<ClusterSpec>),

    /// Return a [`Stream`] of [`ClusterSpecEntry`] objects.
    ClusterSpecEntries(ClusterSpecEntryStream),

    /// Return a [`Stream`] of [`NAction`] objects.
    NActions(NActionStream),

    /// Return a [`Stream`] of [`NActionEntry`] objects.
    NActionEntries(NActionEntryStream),

    /// Return a [`Namespace`], if one was found matching the query.
    Namespace(Option<Namespace>),

    /// Return a [`Stream`] of [`NamespaceEntry`] objects.
    NamespaceEntries(NamespaceEntryStream),

    /// Return a [`Stream`] of [`Node`] objects.
    NodesList(NodesStream),

    /// Return an [`OAction`], if one was found matching the query.
    OAction(Option<OAction>),

    /// Return a [`Stream`] of [`OAction`] objects.
    OActions(OActionStream),

    /// Return a [`Stream`] of [`OActionEntry`] objects.
    OActionEntries(OActionEntryStream),

    /// Return a [`Platform`], if one was found matching the query.
    Platform(Option<Platform>),

    /// Return a [`Stream`] of [`PlatformEntry`] objects.
    PlatformEntries(PlatformEntryStream),

    /// Return a [`Stream`] of [`Shard`] objects.
    ShardsList(ShardsStream),

    /// Return a [`Stream`] of [`StoreExtras`] objects.
    StoreExtrasList(StoreExtrasStream),

    /// Return a [`Stream`] (async iterator) of strings (useful for IDs).
    StringStream(StringStream),
}

// --- Operations return types --- //
/// Alias for a heap-allocated [`Stream`] of cluster spec summaries.
pub type ClusterSpecEntryStream = std::pin::Pin<Box<dyn Stream<Item = Result<ClusterSpecEntry>>>>;

/// Alias for a heap-allocated [`Stream`] of node actions.
pub type NActionStream = std::pin::Pin<Box<dyn Stream<Item = Result<NAction>> + Send>>;

/// Alias for a heap-allocated [`Stream`] of node action summaries.
pub type NActionEntryStream = std::pin::Pin<Box<dyn Stream<Item = Result<NActionEntry>>>>;

/// Alias for a heap-allocated [`Stream`] of namespace summaries.
pub type NamespaceEntryStream = std::pin::Pin<Box<dyn Stream<Item = Result<NamespaceEntry>>>>;

/// Alias for a heap-allocated [`Stream`] of cluster nodes.
pub type NodesStream = std::pin::Pin<Box<dyn Stream<Item = Result<Node>> + Send>>;

/// Alias for a heap-allocated [`Stream`] of orchestrator actions.
pub type OActionStream = std::pin::Pin<Box<dyn Stream<Item = Result<OAction>> + Send>>;

/// Alias for a heap-allocated [`Stream`] of orchestrator action summaries.
pub type OActionEntryStream = std::pin::Pin<Box<dyn Stream<Item = Result<OActionEntry>>>>;

/// Alias for a heap-allocated [`Stream`] of platform summaries.
pub type PlatformEntryStream = std::pin::Pin<Box<dyn Stream<Item = Result<PlatformEntry>>>>;

/// Alias for a heap-allocated [`Stream`] of cluster [`Shard`]s.
pub type ShardsStream = std::pin::Pin<Box<dyn Stream<Item = Result<Shard>> + Send>>;

/// Alias for a heap-allocated [`Stream`] of cluster nodes extras.
pub type StoreExtrasStream = std::pin::Pin<Box<dyn Stream<Item = Result<StoreExtras>> + Send>>;

/// Alias for a heap-allocated [`Stream`] of strings (useful for IDs).
pub type StringStream = std::pin::Pin<Box<dyn Stream<Item = Result<String>>>>;

// --- High level query operations --- //
/// List the summary information of all cluster specs in a namespace, sorted alphabetically.
pub struct ListClusterSpecs(pub NamespaceID);

/// List summary information of all known namespaces, sorted alphabetically.
pub struct ListNamespaces;

/// List all nodes in a cluster, sorted by node ID.
pub struct ListNodes(pub NamespacedResourceID);

impl ListNodes {
    /// List cluster nodes by namespace and cluster ID.
    pub fn by<S1, S2>(ns_id: S1, cluster_id: S2) -> ListNodes
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let id = NamespacedResourceID {
            name: cluster_id.into(),
            ns_id: ns_id.into(),
        };
        ListNodes(id)
    }
}

/// List summary information about known platforms in the namespace, sorted alphabetically.
pub struct ListPlatforms(pub NamespaceID);

/// List store extras for all nodes in a cluster, sorted by node ID.
pub struct ListStoreExtras(pub NamespacedResourceID);

impl ListStoreExtras {
    /// List cluster nodes store extras by namespace and cluster ID.
    pub fn by<S1, S2>(ns_id: S1, cluster_id: S2) -> ListStoreExtras
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let id = NamespacedResourceID {
            name: cluster_id.into(),
            ns_id: ns_id.into(),
        };
        ListStoreExtras(id)
    }
}

/// Lookup a [`ConvergeState`] by namespace and cluster by ID.
#[derive(Clone, Debug)]
pub struct LookupConvergeState(pub NamespacedResourceID);

impl LookupConvergeState {
    /// Lookup a cluster converge state by namespace ID and platform name.
    pub fn by<S1, S2>(ns_id: S1, name: S2) -> LookupConvergeState
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let id = NamespacedResourceID {
            name: name.into(),
            ns_id: ns_id.into(),
        };
        LookupConvergeState(id)
    }
}

impl From<&ClusterSpec> for LookupConvergeState {
    fn from(value: &ClusterSpec) -> Self {
        LookupConvergeState::by(&value.ns_id, &value.cluster_id)
    }
}

/// Lookup a [`ClusterDiscovery`] by namespace and cluster by ID.
#[derive(Clone, Debug)]
pub struct LookupClusterDiscovery(pub NamespacedResourceID);

impl LookupClusterDiscovery {
    /// Lookup a cluster discovery by namespace ID and platform name.
    pub fn by<S1, S2>(ns_id: S1, name: S2) -> LookupClusterDiscovery
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let id = NamespacedResourceID {
            name: name.into(),
            ns_id: ns_id.into(),
        };
        LookupClusterDiscovery(id)
    }
}

/// Lookup a [`ClusterSpec`] namespace and cluster by ID.
#[derive(Clone, Debug)]
pub struct LookupClusterSpec(pub NamespacedResourceID);

impl LookupClusterSpec {
    /// Lookup a ClusterSpec by namespace ID and platform name.
    pub fn by<S1, S2>(ns_id: S1, name: S2) -> LookupClusterSpec
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let id = NamespacedResourceID {
            name: name.into(),
            ns_id: ns_id.into(),
        };
        LookupClusterSpec(id)
    }
}

/// Lookup a [`Namespace`] record by ID.
#[derive(Clone, Debug)]
pub struct LookupNamespace(pub NamespaceID);
impl From<&Namespace> for LookupNamespace {
    fn from(value: &Namespace) -> Self {
        let id = value.id.clone();
        let value = NamespaceID { id };
        LookupNamespace(value)
    }
}
impl From<String> for LookupNamespace {
    fn from(value: String) -> Self {
        let value = NamespaceID { id: value };
        LookupNamespace(value)
    }
}
impl From<&str> for LookupNamespace {
    fn from(value: &str) -> Self {
        let id = value.to_string();
        let value = NamespaceID { id };
        LookupNamespace(value)
    }
}

/// Lookup a [`OAction`] record by ID.
#[derive(Clone, Debug)]
pub struct LookupOAction(pub OActionID);
impl LookupOAction {
    /// Lookup an orchestrator action by ID.
    pub fn by<S1, S2>(ns_id: S1, cluster_id: S2, action_id: Uuid) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let id = OActionID {
            ns_id: ns_id.into(),
            cluster_id: cluster_id.into(),
            action_id,
        };
        LookupOAction(id)
    }
}
impl From<&OAction> for LookupOAction {
    fn from(value: &OAction) -> Self {
        Self::by(&value.ns_id, &value.cluster_id, value.action_id)
    }
}

/// Lookup a [`Platform`] record by ID.
#[derive(Clone, Debug)]
pub struct LookupPlatform(pub NamespacedResourceID);
impl From<&Platform> for LookupPlatform {
    fn from(value: &Platform) -> Self {
        let name = value.name.clone();
        let ns_id = value.ns_id.clone();
        let value = NamespacedResourceID { name, ns_id };
        LookupPlatform(value)
    }
}

impl LookupPlatform {
    /// Lookup a platform by namespace ID and platform name.
    pub fn by<S1, S2>(ns_id: S1, name: S2) -> LookupPlatform
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let id = NamespacedResourceID {
            name: name.into(),
            ns_id: ns_id.into(),
        };
        LookupPlatform(id)
    }
}

/// Iterate over all unfinished node actions for a cluster.
///
/// Actions are returned ordered by creation date (oldest first).
#[derive(Clone, Debug)]
pub struct UnfinishedNAction(pub NamespacedResourceID);

impl UnfinishedNAction {
    /// Lookup a platform by namespace ID and platform name.
    pub fn for_cluster<S1, S2>(ns_id: S1, cluster_id: S2) -> UnfinishedNAction
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let id = NamespacedResourceID {
            name: cluster_id.into(),
            ns_id: ns_id.into(),
        };
        UnfinishedNAction(id)
    }
}

/// Iterate over all unfinished orchestrator actions for a cluster.
///
/// Actions are returned ordered by creation date (oldest first).
#[derive(Clone, Debug)]
pub struct UnfinishedOAction(pub NamespacedResourceID);

impl UnfinishedOAction {
    /// Lookup a platform by namespace ID and platform name.
    pub fn for_cluster<S1, S2>(ns_id: S1, cluster_id: S2) -> UnfinishedOAction
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let id = NamespacedResourceID {
            name: cluster_id.into(),
            ns_id: ns_id.into(),
        };
        UnfinishedOAction(id)
    }
}

// --- Internal implementation details follow --- //
/// Private module to seal implementation details.
mod seal {
    /// Super-trait to seal the [`QueryOp`](super::QueryOp) trait.
    pub trait SealQueryOp {}
}

// --- Implement QueryOp and super traits on types for transparent operations --- //
impl SealQueryOp for ListClusterSpecs {}
impl QueryOp for ListClusterSpecs {
    type Response = ClusterSpecEntryStream;
}
impl From<ListClusterSpecs> for QueryOps {
    fn from(value: ListClusterSpecs) -> Self {
        QueryOps::ListClusterSpecs(value.0)
    }
}

/// List [`NAction`]s for a cluster.
pub struct ListNActions {
    /// The namespace ID the cluster is in.
    pub ns_id: String,

    /// The ID of the cluster the actions are for.
    pub cluster_id: String,

    /// Include finished actions in the results.
    pub include_finished: bool,
}

impl SealQueryOp for ListNActions {}
impl QueryOp for ListNActions {
    type Response = NActionEntryStream;
}
impl From<ListNActions> for QueryOps {
    fn from(value: ListNActions) -> Self {
        QueryOps::ListNActions(value)
    }
}

impl ListNActions {
    /// List [`NAction`] for a cluster by namespace and cluster IDs.
    pub fn by<S1, S2>(ns_id: S1, cluster_id: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        ListNActions {
            ns_id: ns_id.into(),
            cluster_id: cluster_id.into(),
            include_finished: false,
        }
    }

    /// Include finished [`NAction`]s in the list.
    pub fn with_finished(mut self) -> Self {
        self.include_finished = true;
        self
    }

    /// Exclude finished [`NAction`]s from the list.
    pub fn without_finished(mut self) -> Self {
        self.include_finished = false;
        self
    }
}

impl SealQueryOp for ListNamespaces {}
impl QueryOp for ListNamespaces {
    type Response = NamespaceEntryStream;
}
impl From<ListNamespaces> for QueryOps {
    fn from(_: ListNamespaces) -> Self {
        QueryOps::ListNamespaces
    }
}

impl SealQueryOp for ListNodes {}
impl QueryOp for ListNodes {
    type Response = NodesStream;
}
impl From<ListNodes> for QueryOps {
    fn from(value: ListNodes) -> Self {
        QueryOps::ListNodes(value.0)
    }
}

/// List [`OAction`]s for a cluster.
pub struct ListOActions {
    /// The namespace ID the cluster is in.
    pub ns_id: String,

    /// The ID of the cluster the actions are for.
    pub cluster_id: String,

    /// Include finished actions in the results.
    pub include_finished: bool,
}

impl SealQueryOp for ListOActions {}
impl QueryOp for ListOActions {
    type Response = OActionEntryStream;
}
impl From<ListOActions> for QueryOps {
    fn from(value: ListOActions) -> Self {
        QueryOps::ListOActions(value)
    }
}

impl ListOActions {
    /// List [`OAction`] for a cluster by namespace and cluster IDs.
    pub fn by<S1, S2>(ns_id: S1, cluster_id: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        ListOActions {
            ns_id: ns_id.into(),
            cluster_id: cluster_id.into(),
            include_finished: false,
        }
    }

    /// Include finished [`OAction`]s in the list.
    pub fn with_finished(mut self) -> Self {
        self.include_finished = true;
        self
    }

    /// Exclude finished [`OAction`]s from the list.
    pub fn without_finished(mut self) -> Self {
        self.include_finished = false;
        self
    }
}

impl SealQueryOp for ListPlatforms {}
impl QueryOp for ListPlatforms {
    type Response = PlatformEntryStream;
}
impl From<ListPlatforms> for QueryOps {
    fn from(value: ListPlatforms) -> Self {
        QueryOps::ListPlatforms(value.0)
    }
}

/// List [`Shard`]s for a cluster, optionally filtering by node.
pub struct ListShards {
    /// The namespace ID the cluster is in.
    pub ns_id: String,

    /// The ID of the cluster the shards are on.
    pub cluster_id: String,

    /// The ID of the node the shards are on, if set.
    pub node_id: Option<String>,
}

impl SealQueryOp for ListShards {}
impl QueryOp for ListShards {
    type Response = ShardsStream;
}
impl From<ListShards> for QueryOps {
    fn from(value: ListShards) -> Self {
        QueryOps::ListShards(value)
    }
}

impl ListShards {
    /// List [`Shards`] for a cluster by namespace and cluster IDs.
    pub fn by<S1, S2>(ns_id: S1, cluster_id: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        ListShards {
            ns_id: ns_id.into(),
            cluster_id: cluster_id.into(),
            node_id: None,
        }
    }

    /// Filter [`Shard`]s that are on a specific node.
    pub fn with_node_id<S>(mut self, node_id: S) -> Self
    where
        S: Into<String>,
    {
        self.node_id = Some(node_id.into());
        self
    }
}

impl SealQueryOp for ListStoreExtras {}
impl QueryOp for ListStoreExtras {
    type Response = StoreExtrasStream;
}
impl From<ListStoreExtras> for QueryOps {
    fn from(value: ListStoreExtras) -> Self {
        QueryOps::ListStoreExtras(value.0)
    }
}

impl SealQueryOp for LookupConvergeState {}
impl QueryOp for LookupConvergeState {
    type Response = Option<ConvergeState>;
}
impl From<LookupConvergeState> for QueryOps {
    fn from(value: LookupConvergeState) -> Self {
        QueryOps::ClusterConvergeState(value.0)
    }
}

impl SealQueryOp for LookupClusterDiscovery {}
impl QueryOp for LookupClusterDiscovery {
    type Response = Option<ClusterDiscovery>;
}
impl From<LookupClusterDiscovery> for QueryOps {
    fn from(value: LookupClusterDiscovery) -> Self {
        QueryOps::ClusterDiscovery(value.0)
    }
}
impl SealQueryOp for &ClusterDiscovery {}
impl QueryOp for &ClusterDiscovery {
    type Response = Option<ClusterDiscovery>;
}
impl From<&ClusterDiscovery> for QueryOps {
    fn from(value: &ClusterDiscovery) -> Self {
        let id = NamespacedResourceID {
            ns_id: value.ns_id.clone(),
            name: value.cluster_id.clone(),
        };
        QueryOps::ClusterDiscovery(id)
    }
}

impl SealQueryOp for LookupClusterSpec {}
impl QueryOp for LookupClusterSpec {
    type Response = Option<ClusterSpec>;
}
impl From<LookupClusterSpec> for QueryOps {
    fn from(value: LookupClusterSpec) -> Self {
        QueryOps::ClusterSpec(value.0)
    }
}

impl SealQueryOp for LookupNamespace {}
impl QueryOp for LookupNamespace {
    type Response = Option<Namespace>;
}
impl From<LookupNamespace> for QueryOps {
    fn from(value: LookupNamespace) -> Self {
        QueryOps::Namespace(value)
    }
}

impl SealQueryOp for LookupOAction {}
impl QueryOp for LookupOAction {
    type Response = Option<OAction>;
}
impl From<LookupOAction> for QueryOps {
    fn from(value: LookupOAction) -> Self {
        QueryOps::OAction(value)
    }
}

impl SealQueryOp for LookupPlatform {}
impl QueryOp for LookupPlatform {
    type Response = Option<Platform>;
}
impl From<LookupPlatform> for QueryOps {
    fn from(value: LookupPlatform) -> Self {
        QueryOps::Platform(value.0)
    }
}

impl SealQueryOp for UnfinishedNAction {}
impl QueryOp for UnfinishedNAction {
    type Response = NActionStream;
}
impl From<UnfinishedNAction> for QueryOps {
    fn from(value: UnfinishedNAction) -> Self {
        QueryOps::UnfinishedNAction(value.0)
    }
}

impl SealQueryOp for UnfinishedOAction {}
impl QueryOp for UnfinishedOAction {
    type Response = OActionStream;
}
impl From<UnfinishedOAction> for QueryOps {
    fn from(value: UnfinishedOAction) -> Self {
        QueryOps::UnfinishedOAction(value.0)
    }
}

// --- Implement QueryResponses conversions on return types for transparent operations --- //
impl From<QueryResponses> for Option<ConvergeState> {
    fn from(value: QueryResponses) -> Self {
        match value {
            QueryResponses::ClusterConvergeState(state) => state,
            _ => panic!("unexpected result type for the given query operation"),
        }
    }
}
impl From<QueryResponses> for Option<ClusterDiscovery> {
    fn from(value: QueryResponses) -> Self {
        match value {
            QueryResponses::ClusterDiscovery(disc) => disc,
            _ => panic!("unexpected result type for the given query operation"),
        }
    }
}
impl From<QueryResponses> for Option<ClusterSpec> {
    fn from(value: QueryResponses) -> Self {
        match value {
            QueryResponses::ClusterSpec(spec) => spec,
            _ => panic!("unexpected result type for the given query operation"),
        }
    }
}
impl From<QueryResponses> for ClusterSpecEntryStream {
    fn from(value: QueryResponses) -> Self {
        match value {
            QueryResponses::ClusterSpecEntries(stream) => stream,
            _ => panic!("unexpected result type for the given query operation"),
        }
    }
}
impl From<QueryResponses> for NActionStream {
    fn from(value: QueryResponses) -> Self {
        match value {
            QueryResponses::NActions(stream) => stream,
            _ => panic!("unexpected result type for the given query operation"),
        }
    }
}
impl From<QueryResponses> for NActionEntryStream {
    fn from(value: QueryResponses) -> Self {
        match value {
            QueryResponses::NActionEntries(stream) => stream,
            _ => panic!("unexpected result type for the given query operation"),
        }
    }
}
impl From<QueryResponses> for Option<Namespace> {
    fn from(value: QueryResponses) -> Self {
        match value {
            QueryResponses::Namespace(namespace) => namespace,
            _ => panic!("unexpected result type for the given query operation"),
        }
    }
}
impl From<QueryResponses> for NamespaceEntryStream {
    fn from(value: QueryResponses) -> Self {
        match value {
            QueryResponses::NamespaceEntries(stream) => stream,
            _ => panic!("unexpected result type for the given query operation"),
        }
    }
}
impl From<QueryResponses> for NodesStream {
    fn from(value: QueryResponses) -> Self {
        match value {
            QueryResponses::NodesList(stream) => stream,
            _ => panic!("unexpected result type for the given query operation"),
        }
    }
}
impl From<QueryResponses> for Option<OAction> {
    fn from(value: QueryResponses) -> Self {
        match value {
            QueryResponses::OAction(oaction) => oaction,
            _ => panic!("unexpected result type for the given query operation"),
        }
    }
}
impl From<QueryResponses> for OActionStream {
    fn from(value: QueryResponses) -> Self {
        match value {
            QueryResponses::OActions(stream) => stream,
            _ => panic!("unexpected result type for the given query operation"),
        }
    }
}
impl From<QueryResponses> for OActionEntryStream {
    fn from(value: QueryResponses) -> Self {
        match value {
            QueryResponses::OActionEntries(stream) => stream,
            _ => panic!("unexpected result type for the given query operation"),
        }
    }
}
impl From<QueryResponses> for Option<Platform> {
    fn from(value: QueryResponses) -> Self {
        match value {
            QueryResponses::Platform(platform) => platform,
            _ => panic!("unexpected result type for the given query operation"),
        }
    }
}
impl From<QueryResponses> for PlatformEntryStream {
    fn from(value: QueryResponses) -> Self {
        match value {
            QueryResponses::PlatformEntries(stream) => stream,
            _ => panic!("unexpected result type for the given query operation"),
        }
    }
}
impl From<QueryResponses> for ShardsStream {
    fn from(value: QueryResponses) -> Self {
        match value {
            QueryResponses::ShardsList(stream) => stream,
            _ => panic!("unexpected result type for the given query operation"),
        }
    }
}
impl From<QueryResponses> for StoreExtrasStream {
    fn from(value: QueryResponses) -> Self {
        match value {
            QueryResponses::StoreExtrasList(stream) => stream,
            _ => panic!("unexpected result type for the given query operation"),
        }
    }
}
impl From<QueryResponses> for StringStream {
    fn from(value: QueryResponses) -> Self {
        match value {
            QueryResponses::StringStream(stream) => stream,
            _ => panic!("unexpected result type for the given query operation"),
        }
    }
}
