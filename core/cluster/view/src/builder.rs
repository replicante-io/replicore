//! Incrementally build [`ClusterView`] instances.
use std::sync::Arc;

use anyhow::Result;

use replisdk::core::models::cluster::ClusterDiscovery;
use replisdk::core::models::cluster::ClusterSpec;
use replisdk::core::models::oaction::OAction;

use crate::ClusterView;

macro_rules! check_cluster {
    ($builder: expr, $actual: expr) => {
        if $builder.spec.ns_id != $actual.ns_id || $builder.spec.cluster_id != $actual.cluster_id {
            anyhow::bail!(crate::errors::ClusterNotMatch {
                actual_cluster: $actual.cluster_id,
                actual_ns: $actual.ns_id,
                expect_cluster: $builder.spec.cluster_id.clone(),
                expect_ns: $builder.spec.ns_id.clone(),
            })
        }
    };
}

/// Incrementally build [`ClusterView`] instances.
pub struct ClusterViewBuilder {
    discovery: Option<ClusterDiscovery>,
    oactions_unfinished: Vec<Arc<OAction>>,
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
        check_cluster!(self, discovery);
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
            oactions_unfinished: self.oactions_unfinished,
            spec: self.spec,
        }
    }

    /// Initialise a builder for an empty cluster.
    pub(crate) fn new(spec: ClusterSpec) -> ClusterViewBuilder {
        ClusterViewBuilder {
            discovery: None,
            oactions_unfinished: Vec::new(),
            spec,
        }
    }

    /// Return the namespace ID the cluster belongs to.
    pub fn ns_id(&self) -> &str {
        &self.spec.ns_id
    }

    /// Include an unfinished orchestrator action into the view.
    pub fn oaction(&mut self, oaction: OAction) -> Result<&mut Self> {
        check_cluster!(self, oaction);
        if oaction.state.is_final() {
            anyhow::bail!(crate::errors::FinishedOAction {
                ns_id: oaction.ns_id,
                cluster_id: oaction.cluster_id,
                action_id: oaction.action_id,
            })
        }

        let oaction = Arc::new(oaction);
        self.oactions_unfinished.push(oaction);
        Ok(self)
    }
}
