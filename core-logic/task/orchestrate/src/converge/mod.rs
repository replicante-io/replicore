//! Handle converging the known state of the cluster towards the declared version.
use anyhow::Result;
use once_cell::sync::Lazy;

use replisdk::core::models::namespace::Namespace;

use replicore_cluster_models::ConvergeState;
use replicore_cluster_models::OrchestrateMode;
use replicore_cluster_models::OrchestrateReport;
use replicore_cluster_view::ClusterView;
use replicore_context::Context;
use replicore_injector::Injector;

mod constants;
mod node_scale_up;
mod step;

use self::step::ConvergeStep;
use crate::sync::SyncData;

/// Ordered list of cluster convergence steps to perform.
static STEPS: Lazy<Vec<(&'static str, Box<dyn ConvergeStep>)>> = Lazy::new(|| {
    vec![
        ("node-scale-up", Box::new(self::node_scale_up::NodeScaleUp)),
        // TODO: Cluster init check.
        // TODO: Node joining check (how to choose add or join?).
        // TODO: Node scale down check.
        // TODO: Node replacement check.
    ]
});

/// Data for the convergence step of cluster orchestration.
pub struct ConvergeData {
    pub cluster_new: ClusterView,
    pub injector: Injector,
    pub mode: OrchestrateMode,
    pub ns: Namespace,
    pub report: OrchestrateReport,
    pub state: ConvergeState,
}

impl ConvergeData {
    /// Quick access to the Cluster ID being orchestrated.
    pub fn cluster_id(&self) -> &str {
        &self.cluster_new.spec.cluster_id
    }

    /// Convert a [`InitData`] container into a [`ConvergeData`] container.
    pub async fn convert(context: &Context, value: SyncData) -> Result<Self> {
        let cluster_new = value
            .cluster_new
            .into_inner()
            .expect("orchestate task cluster_new lock poisoned")
            .finish();
        let report = value
            .report
            .into_inner()
            .expect("orchestate task report lock poisoned");
        let op = replicore_store::query::LookupConvergeState::from(&value.cluster_current.spec);
        let state = value
            .injector
            .store
            .query(context, op)
            .await?
            .unwrap_or_default();
        let data = ConvergeData {
            cluster_new,
            injector: value.injector,
            mode: value.mode,
            ns: value.ns,
            report,
            state,
        };
        Ok(data)
    }

    /// Quck access to the Namespace ID being orchestated.
    pub fn ns_id(&self) -> &str {
        &self.ns.id
    }
}

/// Process cluster convergence steps in priority order.
pub async fn run(context: &Context, data: &ConvergeData) -> Result<()> {
    let mut new_state = data.state.clone();
    for (_step_id, step) in STEPS.iter() {
        // TODO: add error to the orchestration report but continue.
        step.converge(context, data, &mut new_state).await?;
    }

    // Persist latest converge state updates.
    data.injector.store.persist(context, new_state).await?;
    Ok(())
}
