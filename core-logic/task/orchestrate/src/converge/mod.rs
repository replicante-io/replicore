//! Handle converging the known state of the cluster towards the declared version.
use std::sync::Mutex;
use std::sync::MutexGuard;

use anyhow::Result;
use once_cell::sync::Lazy;

use replisdk::core::models::namespace::Namespace;

use replicore_cluster_models::ConvergeState;
use replicore_cluster_models::OrchestrateMode;
use replicore_cluster_models::OrchestrateReport;
use replicore_cluster_models::OrchestrateReportNote;
use replicore_cluster_view::ClusterView;
use replicore_context::Context;
use replicore_injector::Injector;

mod cluster_init;
mod constants;
mod errors;
mod node_scale_up;
mod step;

use self::step::ConvergeStep;
use crate::sync::SyncData;

/// Ordered list of cluster convergence steps to perform.
static STEPS: Lazy<Vec<(&'static str, Box<dyn ConvergeStep>)>> = Lazy::new(|| {
    vec![
        (
            self::constants::STEP_ID_SCALE_UP,
            Box::new(self::node_scale_up::NodeScaleUp),
        ),
        (
            self::constants::STEP_ID_CLUSTER_INIT,
            Box::new(self::cluster_init::ClusterInit),
        ),
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
    pub report: Mutex<OrchestrateReport>,
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
            .expect("orchestrate task cluster_new lock poisoned")
            .finish();
        let op = replicore_store::query::LookupConvergeState::from(&value.cluster_current.spec);
        let state = value.injector.store.query(context, op).await?;
        let state = match state {
            Some(state) => state,
            None => ConvergeState::clean_state_for(
                &value.cluster_current.spec.ns_id,
                &value.cluster_current.spec.cluster_id,
            ),
        };
        let data = ConvergeData {
            cluster_new,
            injector: value.injector,
            mode: value.mode,
            ns: value.ns,
            report: value.report,
            state,
        };
        Ok(data)
    }

    /// Mutable access to the [`OrchestrateReport`].
    pub fn report_mut(&self) -> MutexGuard<OrchestrateReport> {
        self.report
            .lock()
            .expect("orchestrate task report lock poisoned")
    }

    /// Quick access to the Namespace ID being orchestrated.
    pub fn ns_id(&self) -> &str {
        &self.ns.id
    }
}

/// Process cluster convergence steps in priority order.
pub async fn run(context: &Context, data: &ConvergeData) -> Result<()> {
    let mut new_state = data.state.clone();
    for (step_id, step) in STEPS.iter() {
        let result = step.converge(context, data, &mut new_state).await;
        if let Err(error) = result {
            let message = "Cluster convergence step failed";
            let mut note = OrchestrateReportNote::error(message, error);
            note.data.insert("step-id".into(), (*step_id).into());
            data.report_mut().notes.push(note);
        }
    }

    // Persist latest converge state updates.
    data.injector.store.persist(context, new_state).await?;
    Ok(())
}
