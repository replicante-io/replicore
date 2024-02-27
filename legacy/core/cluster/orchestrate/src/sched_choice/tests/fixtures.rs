use std::str::FromStr;

use anyhow::Result;
use replisdk::platform::models::ClusterDiscovery;

use replicante_models_core::actions::node::ActionState;
use replicante_models_core::actions::node::ActionSyncSummary;
use replicante_models_core::actions::orchestrator::OrchestratorAction as OARecord;
use replicante_models_core::actions::orchestrator::OrchestratorActionScheduleMode;
use replicante_models_core::actions::orchestrator::OrchestratorActionState;
use replicante_models_core::actions::orchestrator::OrchestratorActionSyncSummary;
use replicante_models_core::cluster::ClusterSettings;
use replicore_cluster_view::ClusterView;
use replicore_cluster_view::ClusterViewBuilder;
use replicore_iface_orchestrator_action::registry_entry_factory;
use replicore_iface_orchestrator_action::OrchestratorAction;
use replicore_iface_orchestrator_action::OrchestratorActionRegistryBuilder;
use replicore_iface_orchestrator_action::ProgressChanges;
use replicore_iface_orchestrator_action::TestRegistryClearGuard;

pub const CLUSTER_ID: &str = "colours";
pub const NAMESPACE: &str = "default";

#[derive(Default)]
pub struct ScheduleExclusive {}
impl OrchestratorAction for ScheduleExclusive {
    fn progress(&self, _: &OARecord) -> Result<Option<ProgressChanges>> {
        Ok(None)
    }
}

registry_entry_factory! {
    handler: ScheduleExclusive,
    schedule_mode: OrchestratorActionScheduleMode::Exclusive,
    summary: "action for test",
    timeout: std::time::Duration::from_secs(0),
}

pub fn orchestrator_action_exclusive_pending() -> OrchestratorActionSyncSummary {
    OrchestratorActionSyncSummary {
        cluster_id: CLUSTER_ID.into(),
        action_id: uuid::Uuid::from_str("347db8f1-dab4-401b-8956-04cd0ca25661").unwrap(),
        kind: "unittest.replicante.io/schedule.exclusive".into(),
        state: OrchestratorActionState::PendingSchedule,
    }
}

pub fn orchestrator_action_exclusive_running() -> OrchestratorActionSyncSummary {
    OrchestratorActionSyncSummary {
        cluster_id: CLUSTER_ID.into(),
        action_id: uuid::Uuid::from_str("347db8f1-dab4-401b-8956-04cd0ca25662").unwrap(),
        kind: "unittest.replicante.io/schedule.exclusive".into(),
        state: OrchestratorActionState::Running,
    }
}

pub fn orchestrator_actions_registry() -> TestRegistryClearGuard {
    let mut builder = OrchestratorActionRegistryBuilder::empty();

    builder
        .register(
            "unittest.replicante.io/schedule.exclusive",
            ScheduleExclusive::registry_entry(),
        )
        .expect("schedule.exclusive failed to register");

    builder.build_as_current();
    TestRegistryClearGuard::default()
}

pub fn node_action_pending() -> ActionSyncSummary {
    ActionSyncSummary {
        cluster_id: CLUSTER_ID.into(),
        node_id: "test.node".into(),
        action_id: uuid::Uuid::from_str("0436430c-2b02-624c-2032-570501212b57").unwrap(),
        state: ActionState::PendingSchedule,
    }
}

pub fn node_action_running() -> ActionSyncSummary {
    ActionSyncSummary {
        cluster_id: CLUSTER_ID.into(),
        node_id: "test.node".into(),
        action_id: uuid::Uuid::from_str("0436430c-2b02-624c-2032-570501212b58").unwrap(),
        // Remember: the NEW state is already schedule with the agent.
        state: ActionState::New,
    }
}

pub fn start_view_builder() -> ClusterViewBuilder {
    let discovery = ClusterDiscovery {
        cluster_id: CLUSTER_ID.into(),
        nodes: vec![],
    };
    let settings = ClusterSettings::synthetic(NAMESPACE, CLUSTER_ID);
    ClusterView::builder(settings, discovery).expect("cluster view build should start")
}
