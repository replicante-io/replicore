use failure::ResultExt;
use replisdk::core::models::platform::Platform;
use replisdk::core::models::platform::PlatformDiscoveryOptions;
use replisdk::core::models::platform::PlatformTransport;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use replicante_models_core::api::apply::SCOPE_NS;
use replicante_models_core::api::validate::ErrorsCollection;
use replicante_models_core::events::Event;
use replicante_stream_events::EmitMessage;

use super::appliers::ApplierArgs;
use crate::ErrorKind;
use crate::Result;

/// Validate a platform object and persist it to the DB.
pub fn replicante_io_v0(args: ApplierArgs) -> Result<Value> {
    // Validate request.
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
            "A Platform name must be attached to the request",
        );
    }
    match object.attributes.get("spec") {
        None => errors.collect(
            "MissingAttribute",
            "spec",
            "A Platform object must have a spec definition",
        ),
        Some(spec) => {
            let spec: std::result::Result<PlatformSpec, _> =
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
    let ns_id = object
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
    let spec = object
        .attributes
        .get("spec")
        .expect("validation should have caught this")
        .clone();
    let spec: PlatformSpec =
        serde_json::from_value(spec).expect("validation should have caught this");

    // Persist the platform object to the store and emit relevant events.
    let platform = spec.to_platform(ns_id, name);
    let span = args.span.map(|span| span.context().clone());
    let event = Event::builder()
        .platform()
        .apply(platform.clone());
    let code = event.code();
    let stream_key = event.entity_id().partition_key();
    let event = EmitMessage::with(stream_key, event)
        .with_context(|_| ErrorKind::EventsStreamEmit(code))?
        .trace(span.clone());
    args.events
        .emit(event)
        .with_context(|_| ErrorKind::EventsStreamEmit("Platform"))?;
    args.store
        .persist()
        .platform(platform, span)
        .with_context(|_| ErrorKind::PrimaryStorePersist("Platform"))?;
    Ok(serde_json::json!(null))
}

/// Specification version of a [`Platform`] object.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct PlatformSpec {
    /// Activate/deactivate the Platform integration without removing it.
    ///
    /// When a Platform is deactivated it will NOT be used for cluster discovery
    /// and attempts to provision and deprovision nodes with it will fail.
    #[serde(default = "PlatformSpec::default_active")]
    pub active: bool,

    /// Cluster discovery configuration for the Platform.
    #[serde(default)]
    pub discovery: PlatformDiscoveryOptions,

    /// Platform connection method and options.
    #[serde(flatten)]
    pub transport: PlatformTransport,
}

impl PlatformSpec {
    /// Default activation state of [`PlatformSpec`]s when not specified.
    fn default_active() -> bool {
        true
    }

    /// Convert the apply specification into a full [`Platform`] definition.
    fn to_platform(self, ns_id: String, name: String) -> Platform {
        Platform {
            ns_id,
            name,
            active: self.active,
            discovery: self.discovery,
            transport: self.transport,
        }
    }
}
