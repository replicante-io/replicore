use chrono::Utc;
use failure::ResultExt;
use serde_json::Value;
use uuid::Uuid;

use replicante_models_core::actions::Action;
use replicante_models_core::actions::ActionRequester;
use replicante_models_core::actions::ActionState;
use replicante_models_core::api::apply::SCOPE_CLUSTER;
use replicante_models_core::api::apply::SCOPE_NODE;
use replicante_models_core::api::apply::SCOPE_NS;
use replicante_models_core::api::validate::ErrorsCollection;

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
    match object.attributes.get("action") {
        Some(action) if action.is_object() => match action.get("kind") {
            Some(kind) if kind.is_string() => (),
            Some(_) => errors.collect(
                "TypeError",
                "action.kind",
                "The action kind identifier must be a string",
            ),
            None => errors.collect(
                "MissingAttribute",
                "action.kind",
                "An action kind identifier is required",
            ),
        },
        None => errors.collect(
            "MissingAttribute",
            "action",
            "An action descriptor is required",
        ),
        Some(_) => errors.collect(
            "TypeError",
            "action",
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
        .get("action")
        .expect("validation should have caught this");
    let kind = spec
        .get("kind")
        .expect("validation should have caught this")
        .as_str()
        .expect("validation should have caught this");
    let action_args = spec.get("args").cloned().unwrap_or_else(|| Value::Null);

    // TODO: decode headers from HTTP request.

    let now = Utc::now();
    let action = Action {
        // IDs.
        cluster_id: cluster.to_string(),
        node_id: node.to_string(),
        action_id: Uuid::new_v4(),

        // Attributes.
        args: action_args,
        created_ts: now,
        finished_ts: None,
        headers: Default::default(),
        kind: kind.to_string(),
        refresh_id: 0,
        requester: ActionRequester::CoreApi,
        scheduled_ts: None,
        state: ActionState::PendingSchedule,
        state_payload: None,
    };

    // Store the pending action for later sheduling.
    let span = args.span.map(|span| span.context().clone());
    args.primary_store
        .persist()
        .action(action.clone(), span.clone())
        .with_context(|_| ErrorKind::PrimaryStorePersist("action"))?;
    args.view_store
        .persist()
        .action(action, span)
        .with_context(|_| ErrorKind::ViewStorePersist("action"))?;
    Ok(Value::Null)
}
