//! Apply handlers for the replicate.io/v0 API version.
use actix_web::HttpResponse;
use jsonschema::JSONSchema;
use once_cell::sync::Lazy;

use replisdk::core::models::cluster::ClusterSpec;
use replisdk::core::models::namespace::Namespace;
use replisdk::core::models::namespace::NamespaceStatus;
use replisdk::core::models::platform::Platform;

use replicore_events::Event;
use replicore_store::query::LookupNamespace;

use super::ApplyArgs;

/// API Version identifier in apply requests.
pub const API_VERSION: &str = "replicante.io/v0";

/// ClusterSpec kind identifier in apply requests.
pub const KIND_CLUSTER_SPEC: &str = "clusterspec";

/// Namespace kind identifier in apply requests.
pub const KIND_NAMESPACE: &str = "namespace";

/// Platform kind identifier in apply requests.
pub const KIND_PLATFORM: &str = "platform";

/// Meta-programming to inline JSON Schemas for input validation.
macro_rules! schema {
    ($schema:ident, $source:expr) => {
        static $schema: Lazy<JSONSchema> = Lazy::new(|| {
            let schema = include_str!($source);
            let error = format!("invalid JSON schema for {}", stringify!($schema));
            let schema = serde_json::from_str(schema).expect(&error);
            JSONSchema::compile(&schema).expect(&error)
        });
    };
}

schema!(CLUSTER_SPEC_SCHEMA, "cluster-spec.schema.json");
schema!(NAMESPACE_SCHEMA, "namespace.schema.json");
schema!(PLATFORM_SCHEMA, "platform.schema.json");

/// Apply the object based on its kind.
pub async fn apply(args: ApplyArgs<'_>) -> Result<HttpResponse, crate::api::Error> {
    let kind = args.object.get("kind").unwrap().as_str().unwrap();
    let kind = kind.to_lowercase();
    match kind.as_str() {
        KIND_CLUSTER_SPEC => self::cluster_spec(args).await,
        KIND_NAMESPACE => self::namespace(args).await,
        KIND_PLATFORM => self::platform(args).await,
        _ => panic!("the v0::knows should catch unsupported kinds"),
    }
}

/// Ensure the object's kind is known (and supported).
pub fn knows(kind: &str) -> bool {
    kind == KIND_CLUSTER_SPEC || kind == KIND_NAMESPACE || kind == KIND_PLATFORM
}

/// JSON decode a payload, return a 400 response on error.
fn decode<T>(value: serde_json::Value) -> Result<T, crate::api::Error>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_value(value).map_err(|error| {
        let source = anyhow::anyhow!(error);
        crate::api::Error::bad_request(source)
    })
}

/// Apply a cluster spec object.
async fn cluster_spec(args: ApplyArgs<'_>) -> Result<HttpResponse, crate::api::Error> {
    // Verify & decode the cluster spec.
    CLUSTER_SPEC_SCHEMA
        .validate(args.object)
        .map_err(crate::api::format_json_schema_errors)?;
    let spec = args.object.get("spec").unwrap().clone();
    let cluster: ClusterSpec = decode(spec)?;

    // Apply the cluster spec.
    let event = Event::new_with_payload(super::constants::APPLY_CLUSTER_SPEC, &cluster)?;
    args.injector.events.change(&args.context, event).await?;
    args.injector.store.persist(&args.context, cluster).await?;
    Ok(HttpResponse::Ok().finish())
}

/// Apply a namespace object.
async fn namespace(args: ApplyArgs<'_>) -> Result<HttpResponse, crate::api::Error> {
    // Verify & decode Namespace.
    NAMESPACE_SCHEMA
        .validate(args.object)
        .map_err(crate::api::format_json_schema_errors)?;
    let spec = args.object.get("spec").unwrap().clone();
    let namespace: Namespace = decode(spec)?;

    // For existing namespaces ensure the status transition is valid.
    let lookup = LookupNamespace::from(&namespace);
    let lookup = args.injector.store.query(&args.context, lookup).await?;
    if let Some(lookup) = lookup {
        let from_status = lookup.status;
        let to_status = &namespace.status;
        match from_status {
            NamespaceStatus::Deleted if !matches!(to_status, NamespaceStatus::Deleted) => {
                let error = anyhow::anyhow!("deleted namespaces cannot be change");
                let error = crate::api::Error::bad_request(error);
                return Err(error);
            }
            NamespaceStatus::Deleting if !matches!(to_status, NamespaceStatus::Deleting) => {
                let error = anyhow::anyhow!("deleting namespace status cannot be change");
                let error = crate::api::Error::bad_request(error);
                return Err(error);
            }
            _ => (),
        };
    }

    // Apply the namespace.
    let event = Event::new_with_payload(super::constants::APPLY_NAMESPACE, &namespace)?;
    args.injector.events.change(&args.context, event).await?;
    args.injector
        .store
        .persist(&args.context, namespace)
        .await?;
    Ok(HttpResponse::Ok().finish())
}

/// Apply a platform object.
async fn platform(args: ApplyArgs<'_>) -> Result<HttpResponse, crate::api::Error> {
    // Verify & decode the platform.
    PLATFORM_SCHEMA
        .validate(args.object)
        .map_err(crate::api::format_json_schema_errors)?;
    let spec = args.object.get("spec").unwrap().clone();
    let platform: Platform = decode(spec)?;

    // Apply the platform.
    let event = Event::new_with_payload(super::constants::APPLY_PLATFORM, &platform)?;
    args.injector.events.change(&args.context, event).await?;
    args.injector.store.persist(&args.context, platform).await?;
    Ok(HttpResponse::Ok().finish())
}
