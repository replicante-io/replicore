//! Apply handlers for the replicate.io/v0 API version.
use actix_web::HttpResponse;
use jsonschema::JSONSchema;
use once_cell::sync::Lazy;

pub mod constants;

mod cluster;
mod namespace;
mod platform;

use super::ApplyArgs;

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
        constants::KIND_CLUSTER_SPEC => self::cluster::cluster_spec(args).await,
        constants::KIND_NAMESPACE => self::namespace::namespace(args).await,
        constants::KIND_PLATFORM => self::platform::platform(args).await,
        _ => panic!("the v0::knows should catch unsupported kinds"),
    }
}

/// Ensure the object's kind is known (and supported).
pub fn knows(kind: &str) -> bool {
    kind == constants::KIND_CLUSTER_SPEC
        || kind == constants::KIND_NAMESPACE
        || kind == constants::KIND_PLATFORM
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
