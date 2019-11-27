use serde_json::Value;

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

    // Convert into usable models.
    let ns = object
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
    let action = object
        .attributes
        .get("action")
        .expect("validation should have caught this");
    let kind = action
        .get("kind")
        .expect("validation should have caught this")
        .as_str()
        .expect("validation should have caught this");

    // TODO: schedule the action
    println!("~~~ ns={};cluster={};node={}", ns, cluster, node);
    println!("~~~ kind={}", kind);
    Ok(Value::Null)
}
