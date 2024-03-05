//! Incrementally build [`ClusterView`] instances.
use anyhow::Result;

use replisdk::core::models::cluster::ClusterDiscovery;
use replisdk::core::models::cluster::ClusterSpec;

use crate::ClusterView;

/// Incrementally build [`ClusterView`] instances.
pub struct ClusterViewBuilder {
    discovery: Option<ClusterDiscovery>,
    spec: ClusterSpec,
}

impl ClusterViewBuilder {
    /// Return the cluster ID the view is about.
    pub fn cluster_id(&self) -> &str {
        &self.spec.cluster_id
    }

    /// Update the view with the given discovery record.
    ///
    /// The discovery is checked to make sure it references the same cluster (namespace and ID).
    pub fn discovery(&mut self, discovery: ClusterDiscovery) -> Result<&mut Self> {
        if self.spec.ns_id != discovery.ns_id || self.spec.cluster_id != discovery.cluster_id {
            anyhow::bail!(crate::errors::ClusterNotMatch {
                actual_cluster: discovery.cluster_id,
                actual_ns: discovery.ns_id,
                expect_cluster: self.spec.cluster_id.clone(),
                expect_ns: self.spec.ns_id.clone(),
            })
        }
        self.discovery = Some(discovery);
        Ok(self)
    }

    /// Complete the building process and return a [`ClusterView`].
    pub fn finish(self) -> ClusterView {
        let discovery = self.discovery.unwrap_or_else(|| ClusterDiscovery {
            ns_id: self.spec.ns_id.clone(),
            cluster_id: self.spec.cluster_id.clone(),
            nodes: Default::default(),
        });
        ClusterView {
            discovery,
            spec: self.spec,
        }
    }

    /// Initialise a builder for an empty cluster.
    pub(crate) fn new(spec: ClusterSpec) -> ClusterViewBuilder {
        ClusterViewBuilder {
            discovery: None,
            spec,
        }
    }

    /// Return the namespace ID the cluster belongs to.
    pub fn ns_id(&self) -> &str {
        &self.spec.ns_id
    }
}
