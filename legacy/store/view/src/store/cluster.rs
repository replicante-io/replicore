use opentracingrust::SpanContext;

use replicante_models_core::cluster::OrchestrateReport;

use crate::backend::ClusterImpl;
use crate::Result;

pub struct Cluster<'query> {
    attrs: ClusterAttributes<'query>,
    cluster: ClusterImpl,
}

impl<'query> Cluster<'query> {
    pub fn new(cluster: ClusterImpl, attrs: ClusterAttributes<'query>) -> Cluster<'query> {
        Cluster { attrs, cluster }
    }

    /// Fetch a specific cluster `OrchestrateReport`.
    pub fn orchestrate_report<S>(&self, span: S) -> Result<Option<OrchestrateReport>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.cluster.orchestrate_report(&self.attrs, span.into())
    }
}

pub struct ClusterAttributes<'query> {
    pub cluster_id: &'query str,
    pub namespace: &'query str,
}

impl<'query> ClusterAttributes<'query> {
    pub fn new(namespace: &'query str, cluster_id: &'query str) -> ClusterAttributes<'query> {
        ClusterAttributes {
            cluster_id,
            namespace,
        }
    }
}
