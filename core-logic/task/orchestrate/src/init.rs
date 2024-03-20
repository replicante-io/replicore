//! Cluster orchestration initialisation steps.
use anyhow::Result;

use replisdk::core::models::namespace::Namespace;
use replisdk::core::models::namespace::NamespaceStatus;

use replicore_cluster_models::OrchestrateMode;
use replicore_cluster_models::OrchestrateReport;
use replicore_cluster_view::ClusterView;
use replicore_context::Context;
use replicore_errors::ClusterNotActive;
use replicore_errors::ClusterNotFound;
use replicore_errors::NamespaceNotActive;
use replicore_errors::NamespaceNotFound;
use replicore_injector::Injector;
use replicore_store::query::LookupClusterSpec;
use replicore_store::query::LookupNamespace;

use crate::OrchestrateCluster;

/// Initial data for cluster orchestration.
pub struct InitData {
    pub cluster_current: ClusterView,
    pub injector: Injector,
    pub mode: OrchestrateMode,
    pub ns: Namespace,
    pub report: OrchestrateReport,
}

impl std::fmt::Debug for InitData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InitData")
            .field("cluster_current", &self.cluster_current)
            .field("injector", &"Injector { ... }")
            .field("ns", &self.ns)
            .finish()
    }
}

impl InitData {
    /// Fetch initial data from the store.
    pub async fn load(
        context: &Context,
        injector: Injector,
        request: OrchestrateCluster,
    ) -> Result<InitData> {
        // Lookup the namespace and ensure it is active.
        let op = LookupNamespace::from(request.ns_id.clone());
        let ns = match injector.store.query(context, op).await? {
            Some(ns) => ns,
            None => anyhow::bail!(NamespaceNotFound::new(request.ns_id)),
        };
        if matches!(ns.status, NamespaceStatus::Inactive) {
            anyhow::bail!(NamespaceNotActive::new(request.ns_id));
        }

        // Lookup the ClusterSpec and check its status.
        let op = LookupClusterSpec::by(&request.ns_id, &request.cluster_id);
        let spec = match injector.store.query(context, op).await? {
            Some(spec) => spec,
            None => anyhow::bail!(ClusterNotFound::new(request.ns_id, request.cluster_id)),
        };
        if !spec.active {
            anyhow::bail!(ClusterNotActive::new(request.ns_id, request.cluster_id));
        }

        // Load existing cluster view.
        let cluster_current = ClusterView::load(context, &injector.store, spec)
            .await?
            .finish();

        let mode = match ns.status {
            NamespaceStatus::Deleted | NamespaceStatus::Deleting => OrchestrateMode::Delete,
            NamespaceStatus::Observed => OrchestrateMode::Observe,
            NamespaceStatus::Inactive => panic!("inactive namespaces were rejected earlier"),
            NamespaceStatus::Active => OrchestrateMode::Sync,
        };
        let report = OrchestrateReport::start(&ns.id, &cluster_current.spec.cluster_id, mode);

        // Collect all initial data and return it.
        let data = InitData {
            cluster_current,
            injector,
            mode,
            ns,
            report,
        };
        Ok(data)
    }
}
