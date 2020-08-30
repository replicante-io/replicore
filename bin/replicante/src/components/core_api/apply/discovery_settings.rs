use failure::ResultExt;
use serde_json::Value;

use replicante_models_core::api::apply::SCOPE_NS;
use replicante_models_core::api::objects::DiscoverySettings as DiscoverySettingsObject;
use replicante_models_core::api::validate::ErrorsCollection;
use replicante_models_core::cluster::discovery::DiscoverySettings;
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
    if object.metadata.get(SCOPE_NS).is_none() {
        errors.collect(
            "MissingAttribute",
            format!("metadata.{}", SCOPE_NS),
            "A namespace id must be attached to the request",
        );
    }
    if object.metadata.get("name").is_none() {
        errors.collect(
            "MissingAttribute",
            "metadata.name",
            "A DiscoverySetting name must be attached to the request",
        );
    }
    match object.attributes.get("spec") {
        None => errors.collect(
            "MissingAttribute",
            "spec",
            "A DiscoverySetting object must have a spec definition",
        ),
        Some(spec) => {
            let spec: std::result::Result<DiscoverySettingsObject, _> =
                serde_json::from_value(spec.clone());
            if let Err(error) = spec {
                errors.collect(
                    "InvalidAttribute",
                    "spec",
                    format!("Invalid specification: {}", error),
                );
            }
        }
    }
    errors.into_result(ErrorKind::ValidateFailed)?;

    // Convert ApplierArgs into a usable structures.
    let ns = object
        .metadata
        .get(SCOPE_NS)
        .expect("validation should have caught this")
        .as_str()
        .expect("validation should have caught this")
        .to_string();
    let name = object
        .metadata
        .get("name")
        .expect("validation should have caught this")
        .as_str()
        .expect("validation should have caught this")
        .to_string();
    let settings = object
        .attributes
        .get("spec")
        .expect("validation should have caught this")
        .clone();
    let settings: DiscoverySettingsObject =
        serde_json::from_value(settings).expect("validation should have caught this");

    // Persist the settings to the DB and emit relevant events.
    let settings = DiscoverySettings::from_object(ns, name, settings);
    let span = args.span.map(|span| span.context().clone());
    let event = Event::builder()
        .namespace()
        .discovery_settings(settings.clone());
    let code = event.code();
    let stream_key = event.stream_key();
    let event = EmitMessage::with(stream_key, event)
        .with_context(|_| ErrorKind::EventsStreamEmit(code))?
        .trace(span.clone());
    args.events
        .emit(event)
        .with_context(|_| ErrorKind::EventsStreamEmit("DiscoverySettings"))?;
    args.store
        .persist()
        .discovery_settings(settings, span)
        .with_context(|_| ErrorKind::PrimaryStorePersist("DiscoverySettings"))?;
    Ok(serde_json::json!(null))
}
