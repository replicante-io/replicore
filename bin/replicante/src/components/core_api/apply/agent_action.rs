use chrono::Utc;
use failure::ResultExt;
use serde_json::Value;
use uuid::Uuid;

use replicante_models_core::actions::Action;
use replicante_models_core::actions::ActionApproval;
use replicante_models_core::actions::ActionRequester;
use replicante_models_core::actions::ActionState;
use replicante_models_core::api::apply::SCOPE_CLUSTER;
use replicante_models_core::api::apply::SCOPE_NODE;
use replicante_models_core::api::apply::SCOPE_NS;
use replicante_models_core::api::validate::ErrorsCollection;
use replicante_models_core::events::Event;
use replicante_stream_events::EmitMessage;

use super::appliers::ApplierArgs;
use crate::ErrorKind;
use crate::Result;

/// Validate an action request and add it to the DB.
pub fn replicante_io_v0(args: ApplierArgs) -> Result<Value> {
    // Valiate request.
    let mut errors = ErrorsCollection::new();
    let object = &args.object;
    if object.metadata.get(SCOPE_CLUSTER).is_none() {
        errors.collect(
            "MissingAttribute",
            format!("metadata.{}", SCOPE_CLUSTER),
            "A cluster id must be attached to the request",
        );
    }
    if object.metadata.get(SCOPE_NODE).is_none() {
        errors.collect(
            "MissingAttribute",
            format!("metadata.{}", SCOPE_NODE),
            "A node id must be attached to the request",
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
        Some(spec) if spec.is_object() => match spec.get("action") {
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
        },
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
    let node = object
        .metadata
        .get(SCOPE_NODE)
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

    let action_args = spec.get("args").cloned().unwrap_or_else(|| Value::Null);
    let approval = match object.metadata.get("approval") {
        None => ActionApproval::default(),
        Some(approval) => {
            serde_json::from_value(approval.clone()).expect("validation should have caught this")
        }
    };
    let state = match approval {
        ActionApproval::Granted => ActionState::PendingSchedule,
        ActionApproval::Required => ActionState::PendingApprove,
    };

    let now = Utc::now();
    let action = Action {
        action_id: Uuid::new_v4(),
        args: action_args,
        cluster_id: cluster.to_string(),
        created_ts: now,
        finished_ts: None,
        headers: args.headers,
        kind: kind.to_string(),
        node_id: node.to_string(),
        refresh_id: 0,
        requester: ActionRequester::CoreApi,
        schedule_attempt: 0,
        scheduled_ts: None,
        state,
        state_payload: None,
    };

    // Store the pending action for later sheduling.
    let span = args.span.map(|span| span.context().clone());
    let event = Event::builder().action().new_action(action.clone());
    let stream_key = event.stream_key();
    let event = EmitMessage::with(stream_key, event)
        .with_context(|_| ErrorKind::EventsStreamEmit("ACTION_NEW"))?
        .trace(span.clone());
    args.events
        .emit(event)
        .with_context(|_| ErrorKind::EventsStreamEmit("ACTION_NEW"))?;
    args.store
        .persist()
        .action(action, span)
        .with_context(|_| ErrorKind::PrimaryStorePersist("action"))?;
    Ok(Value::Null)
}
