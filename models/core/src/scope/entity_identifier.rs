use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

/// Identifies a specific entity in the system.
///
/// These `EntityId`s can be used to references entities such as actions, clusters,
/// namespaces and more whenever such entities are part of an event, context or similar.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum EntityId<'entity> {
    /// Represents Replicante Core as a system as a whole.
    System,

    /// Identifies a namespace by ID.
    Namespace(&'entity str),

    /// Identifies a managed cluster by (Namespace, Cluster) ID.
    Cluster(&'entity str, &'entity str),

    /// Identifies an action targeting a managed cluster by (Namespace, Cluster, Action) ID.
    ClusterAction(&'entity str, &'entity str, Uuid),

    /// Identifies a managed cluster node by (Namespace, Cluster, Node) ID.
    Node(&'entity str, &'entity str, &'entity str),

    /// Identifies an action targeting a managed cluster node by (Namespace, Cluster, Node, Action) ID.
    NodeAction(&'entity str, &'entity str, &'entity str, Uuid),

    /// Identifies a managed cluster data shard by (Namespace, Cluster, Node, Shard) ID.
    Shard(&'entity str, &'entity str, &'entity str, &'entity str),
}

impl<'entity> EntityId<'entity> {
    /// Interpret cluster sub-entities such as nodes as clusters.
    ///
    /// This has no effect on system and namespace entities.
    pub fn as_cluster(&self) -> EntityId {
        match self {
            Self::System => Self::System,
            Self::Namespace(ns) => Self::Namespace(ns),
            Self::Cluster(ns, cluster) => Self::Cluster(ns, cluster),
            Self::ClusterAction(ns, cluster, _) => Self::Cluster(ns, cluster),
            Self::Node(ns, cluster, _) => Self::Cluster(ns, cluster),
            Self::NodeAction(ns, cluster, _, _) => Self::Cluster(ns, cluster),
            Self::Shard(ns, cluster, _, _) => Self::Cluster(ns, cluster),
        }
    }

    /// Key to partition entities such that entities relating to the same logical object.
    ///
    /// * All cluster entities relating to the same cluster are grouped together.
    /// * Namespace entities are grouped by namespaces.
    /// * Global entities have their own group too.
    pub fn partition_key(&self) -> String {
        // NOTE: the use of <>, [] and () is to prevent entities names from clashing.
        //       For example a namespace called `default.test` would clash with cluster `test` in
        //       the `default` namespace without them.
        match self {
            Self::System => "<replicore>".to_string(),
            Self::Namespace(ns) => format!("[{ns}]"),
            Self::Cluster(ns, cluster) => format!("[{ns}].({cluster})"),
            Self::ClusterAction(ns, cluster, _) => format!("[{ns}].({cluster})"),
            Self::Node(ns, cluster, _) => format!("[{ns}].({cluster})"),
            Self::NodeAction(ns, cluster, _, _) => format!("[{ns}].({cluster})"),
            Self::Shard(ns, cluster, _, _) => format!("[{ns}].({cluster})"),
        }
    }
}

impl<'entity> std::fmt::Display for EntityId<'entity> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "<replicore>"),
            Self::Namespace(ns) => write!(f, "{}", ns),
            Self::Cluster(ns, cluster) => write!(f, "{}.{}", ns, cluster),
            Self::ClusterAction(ns, cluster, action) => {
                write!(f, "{}.{}/action={}", ns, cluster, action)
            }
            Self::Node(ns, cluster, node) => write!(f, "{}.{}/node={}", ns, cluster, node),
            Self::NodeAction(ns, cluster, node, action) => {
                write!(f, "{}.{}/node={}/action={}", ns, cluster, node, action)
            }
            Self::Shard(ns, cluster, node, shard) => {
                write!(f, "{}.{}/node={}/shard={}", ns, cluster, node, shard)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EntityId;

    #[test]
    fn partition_key_for_system() {
        let entity = EntityId::System;
        assert_eq!(entity.partition_key(), "<replicore>");
    }

    #[test]
    fn partition_key_for_namespace() {
        let entity = EntityId::Namespace("default");
        assert_eq!(entity.partition_key(), "[default]");
    }

    #[test]
    fn partition_key_for_cluster() {
        let entity = EntityId::Cluster("default", "test");
        assert_eq!(entity.partition_key(), "[default].(test)");
    }

    #[test]
    fn partition_key_limits_clashes() {
        let entity = EntityId::Cluster("", "default.test");
        assert_eq!(entity.partition_key(), "[].(default.test)");

        let entity = EntityId::Cluster("ns].(cluster)[", "default.test");
        assert_eq!(entity.partition_key(), "[ns].(cluster)[].(default.test)");
    }
}
