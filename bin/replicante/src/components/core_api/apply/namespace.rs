use failure::ResultExt;
use serde_json::Value;

use replicante_models_core::api::validate::ErrorsCollection;
use replicante_models_core::events::Event;
use replicante_models_core::scope::Namespace;
use replicante_models_core::scope::NsHttpsTransport;
use replicante_stream_events::EmitMessage;

use super::appliers::ApplierArgs;
use crate::ErrorKind;
use crate::Result;

/// Validate a namespace request and add it to the store.
pub fn replicante_io_v0(args: ApplierArgs) -> Result<Value> {
    // Validate request.
    let mut errors = ErrorsCollection::new();
    let object = &args.object;
    if object.metadata.get("name").is_none() {
        errors.collect(
            "MissingAttribute",
            "metadata.name",
            "A Namespace name must be attached to the request",
        );
    }
    // Check Namespace options are valid (if set).
    match object.attributes.get("spec") {
        None => (),
        Some(spec) => {
            // Check https_transport options are valid (if set).
            match spec.get("https_transport") {
                None => (),
                Some(https_transport) => {
                    let https_transport: std::result::Result<NsHttpsTransport, _> =
                        serde_json::from_value(https_transport.clone());
                    if let Err(error) = https_transport {
                        errors.collect(
                            "InvalidAttribute",
                            "spec.https_transport",
                            format!("Invalid HTTPS Transport specification: {}", error),
                        );
                    }
                }
            }
        }
    }
    errors.into_result(ErrorKind::ValidateFailed)?;

    // Convert ApplierArgs into usable structures.
    let ns_id = object
        .metadata
        .get("name")
        .expect("validation should have caught this")
        .as_str()
        .expect("validation should have caught this")
        .to_string();
    let https_transport = object
        .attributes
        .get("spec")
        .and_then(|spec| spec.get("https_transport"))
        .cloned()
        .map(|https_transport| {
            serde_json::from_value(https_transport).expect("validation should have caught this")
        })
        .unwrap_or_default();

    // TODO(namespace-rollout): Remove this and allow any namespace ID.
    if ns_id != Namespace::HARDCODED_FOR_ROLLOUT().ns_id {
        let error = ErrorKind::NamespaceRolloutNotDefault(ns_id);
        return Err(error.into());
    }

    // Persist the namespace to the store and emit relevant events.
    let namespace = Namespace { ns_id, https_transport };
    let span = args.span.map(|span| span.context().clone());
    let event = Event::builder()
        .namespace()
        .apply(namespace.clone());
    let code = event.code();
    let stream_key = event.entity_id().partition_key();
    let event = EmitMessage::with(stream_key, event)
        .with_context(|_| ErrorKind::EventsStreamEmit(code))?
        .trace(span.clone());
    args.events
        .emit(event)
        .with_context(|_| ErrorKind::EventsStreamEmit("Namespace"))?;
    args.store
        .persist()
        .namespace(namespace, span)
        .with_context(|_| ErrorKind::PrimaryStorePersist("Namespace"))?;
    Ok(serde_json::json!(null))
}
