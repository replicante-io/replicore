use opentracingrust::SpanContext;

use replicante_models_core::cluster::discovery::ClusterDiscovery;
use replicante_models_core::cluster::ClusterSettings;

use crate::backend::ClusterImpl;
use crate::Result;

/// Operate on cluster-level models.
pub struct Cluster {
    cluster: ClusterImpl,
    attrs: ClusterAttributes,
}

impl Cluster {
    pub(crate) fn new(cluster: ClusterImpl, attrs: ClusterAttributes) -> Cluster {
        Cluster { cluster, attrs }
    }

    /// Query a `ClusterDiscovery` record, if any is stored.
    pub fn discovery<S>(&self, span: S) -> Result<Option<ClusterDiscovery>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.cluster.discovery(&self.attrs, span.into())
    }

    /// Query a `ClusterSettings` record, if any is stored.
    pub fn settings<S>(&self, span: S) -> Result<Option<ClusterSettings>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.cluster.settings(&self.attrs, span.into())
    }
}

/// Attributes attached to all cluster-level operations.
pub struct ClusterAttributes {
    pub cluster_id: String,
    pub namespace: String,
}
