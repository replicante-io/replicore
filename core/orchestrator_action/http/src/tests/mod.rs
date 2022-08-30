use std::collections::HashMap;

use uuid::Uuid;

use replicante_models_core::actions::orchestrator::OrchestratorAction;
use replicante_models_core::actions::orchestrator::OrchestratorActionState;

mod args;
mod response;

pub const CLUSTER_ID: &str = "colours";

lazy_static::lazy_static! {
    pub static ref UUID1: Uuid = "a7514ce6-48f4-4f9d-bb22-78cbfc37c664".parse().unwrap();
}

fn action_record() -> OrchestratorAction {
    let args = serde_json::json!({
        "remote": {
            "url": "https://test.local:1234",
        },
    });
    OrchestratorAction {
        cluster_id: CLUSTER_ID.to_string(),
        action_id: *UUID1,
        args,
        created_ts: chrono::Utc::now(),
        finished_ts: None,
        headers: HashMap::new(),
        kind: "core.replicante.io/debug.counting".into(),
        scheduled_ts: None,
        state: OrchestratorActionState::Running,
        state_payload: None,
        state_payload_error: None,
        timeout: None,
    }
}
