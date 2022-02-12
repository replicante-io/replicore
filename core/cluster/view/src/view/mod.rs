use anyhow::anyhow;
use anyhow::Result;

use replicante_models_core::cluster::discovery::ClusterDiscovery;
use replicante_models_core::cluster::ClusterSettings;

use crate::ClusterViewCorrupt;

#[cfg(test)]
mod tests;

/// Syntetic in-memory view of a Cluster.
pub struct ClusterView {
    // Cluster identification attributes
    pub cluster_id: String,
    pub namespace: String,

    // Whole cluster records.
    pub discovery: ClusterDiscovery,
    pub settings: ClusterSettings,
}

impl ClusterView {
    /// Start building a Cluster view from essential data.
    pub fn builder(
        settings: ClusterSettings,
        discovery: ClusterDiscovery,
    ) -> Result<ClusterViewBuilder> {
        // Grab the namespace and cluster id from the settings.
        let namespace = settings.namespace.clone();
        let cluster_id = settings.cluster_id.clone();

        // Then make sure the discovery record is for the same cluster.
        // TODO(namespace-rollout): Validate namespaces match.
        if discovery.cluster_id != cluster_id {
            return Err(anyhow!(ClusterViewCorrupt::cluster_id_clash(
                namespace,
                cluster_id,
                discovery.cluster_id
            )));
        }

        let view = ClusterView {
            cluster_id,
            namespace,
            discovery,
            settings,
        };
        Ok(ClusterViewBuilder { view })
    }
}

/// Incrementally build a Cluster view from individual records.
pub struct ClusterViewBuilder {
    view: ClusterView,
}

impl ClusterViewBuilder {
    /// Convert this view builder into a complete ClusterView.
    pub fn build(self) -> ClusterView {
        self.view
    }
}
