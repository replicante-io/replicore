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
use crate::init::InitData;

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
    pub cluster_current: ClusterView,
    pub injector: Injector,
    pub mode: OrchestrateMode,
    pub ns: Namespace,
    pub report: OrchestrateReport,
    pub state: ConvergeState,
}

impl ConvergeData {
    /// Convert a [`InitData`] container into a [`ConvergeData`] container.
    pub async fn convert(context: &Context, value: InitData) -> Result<Self> {
        let op = replicore_store::query::LookupConvergeState::from(&value.cluster_current.spec);
        let state = value
            .injector
            .store
            .query(context, op)
            .await?
            .unwrap_or_default();
        let data = ConvergeData {
            cluster_current: value.cluster_current,
            injector: value.injector,
            mode: value.mode,
            ns: value.ns,
            report: value.report,
            state,
        };
        Ok(data)
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
