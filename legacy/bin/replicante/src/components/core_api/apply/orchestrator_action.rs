use std::time::Duration;

use chrono::Utc;
use failure::ResultExt;
use serde_json::Value;
use uuid::Uuid;

use replicante_models_core::actions::orchestrator::OrchestratorAction;
use replicante_models_core::actions::orchestrator::OrchestratorActionState;
use replicante_models_core::actions::ActionApproval;
use replicante_models_core::api::apply::SCOPE_CLUSTER;
use replicante_models_core::api::apply::SCOPE_NS;
use replicante_models_core::api::validate::ErrorsCollection;
use replicante_models_core::events::Event;
use replicante_stream_events::EmitMessage;

use super::appliers::ApplierArgs;
use crate::ErrorKind;
use crate::Result;

/// Validate an orchestrator action request and add it to the DB.
pub fn replicante_io_v0(args: ApplierArgs) -> Result<Value> {
    // Validate request.
    let mut errors = ErrorsCollection::new();
    let object = &args.object;
    if object.metadata.get(SCOPE_CLUSTER).is_none() {
        errors.collect(
            "MissingAttribute",
            format!("metadata.{}", SCOPE_CLUSTER),
            "A cluster id must be attached to the request",
        );
    }
    if object.metadata.get(SCOPE_NS).is_none() {
        errors.collect(
            "MissingAttribute",
            format!("metadata.{}", SCOPE_NS),
            "A namespace id must be attached to the request",
        );
    }
    match object.metadata.get("approval") {
        None => (),
        Some(approval) if approval.is_string() => {
            let approval: std::result::Result<ActionApproval, _> =
                serde_json::from_value(approval.clone());
            if let Err(error) = approval {
                errors.collect(
                    "InvalidAttribute",
                    "metadata.approval",
                    format!("Invalid approval: {}", error),
                );
            }
        }
        _ => errors.collect(
            "TypeError",
            "metadata.approval",
            "Action approval attribute must be a string",
        ),
    }
    match object.attributes.get("spec") {
        Some(spec) if spec.is_object() => {
            // Ensure action is set and valid.
            match spec.get("action") {
                Some(kind) if kind.is_string() => (),
                Some(_) => errors.collect(
                    "TypeError",
                    "spec.action",
                    "The action kind identifier must be a string",
                ),
                None => errors.collect(
                    "MissingAttribute",
                    "spec.action",
                    "An action kind identifier is required",
                ),
            };
            // Ensure timeout is a Duration, if set.
            if let Some(timeout) = spec.get("timeout").cloned() {
                if let Err(error) = serde_json::from_value::<Duration>(timeout) {
                    errors.collect(
                        "InvalidAttribute",
                        "spec.timeout",
                        format!("Invalid timeout: {}", error),
                    );
                }
            }
        }
        None => errors.collect(
            "MissingAttribute",
            "spec",
            "An action descriptor is required",
        ),
        Some(_) => errors.collect(
            "TypeError",
            "spec",
            "The action descriptor must be an object",
        ),
    }
    errors.into_result(ErrorKind::ValidateFailed)?;

    // Convert ApplierArgs into a usable Action model.
    let _ns = object
        .metadata
        .get(SCOPE_NS)
        .expect("validation should have caught this")
        .as_str()
        .expect("validation should have caught this");
    let cluster = object
        .metadata
        .get(SCOPE_CLUSTER)
        .expect("validation should have caught this")
        .as_str()
        .expect("validation should have caught this");
    let spec = object
        .attributes
        .get("spec")
        .expect("validation should have caught this");
    let kind = spec
        .get("action")
        .expect("validation should have caught this")
        .as_str()
        .expect("validation should have caught this");

    let action_args = spec.get("args").cloned().unwrap_or(Value::Null);
    let approval = match object.metadata.get("approval") {
        None => ActionApproval::default(),
        Some(approval) => {
            serde_json::from_value(approval.clone()).expect("validation should have caught this")
        }
    };
    let state = match approval {
        ActionApproval::Granted => OrchestratorActionState::PendingSchedule,
        ActionApproval::Required => OrchestratorActionState::PendingApprove,
    };
    let timeout = match spec.get("timeout") {
        None => None,
        Some(timeout) => {
            serde_json::from_value(timeout.clone()).expect("validation should have caught this")
        }
    };

    let now = Utc::now();
    let action = OrchestratorAction {
        action_id: Uuid::new_v4(),
        args: action_args,
        cluster_id: cluster.to_string(),
        created_ts: now,
        finished_ts: None,
        headers: args.headers,
        kind: kind.to_string(),
        scheduled_ts: None,
        state,
        state_payload: None,
        state_payload_error: None,
        timeout,
    };

    // Store the pending action for later scheduling.
    let span = args.span.map(|span| span.context().clone());
    let event = Event::builder()
        .action()
        .new_orchestrator_action(action.clone());
    let stream_key = event.entity_id().partition_key();
    let event = EmitMessage::with(stream_key, event)
        .with_context(|_| ErrorKind::EventsStreamEmit("ACTION_ORCHESTRATOR_NEW"))?
        .trace(span.clone());
    args.events
        .emit(event)
        .with_context(|_| ErrorKind::EventsStreamEmit("ACTION_ORCHESTRATOR_NEW"))?;
    args.store
        .persist()
        .orchestrator_action(action, span)
        .with_context(|_| ErrorKind::PrimaryStorePersist("orchestrator action"))?;
    Ok(Value::Null)
}
