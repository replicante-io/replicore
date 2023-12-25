//! Handle API requests to apply (create or update) objects.
use actix_web::web::Data;
use actix_web::web::Json;
use actix_web::HttpResponse;
use jsonschema::JSONSchema;
use once_cell::sync::Lazy;

use replicore_context::Context;
use replicore_injector::Injector;

mod constants;
mod v0;

/// Arguments to pass around apply handlers.
pub struct ApplyArgs<'a> {
    /// Request context for the apply operation.
    #[allow(dead_code)]
    context: Context,

    /// Process dependency injector used during the apply operation.
    injector: Data<Injector>,

    /// Object to apply.
    object: &'a serde_json::Value,
}

/// Compiled JSON schema for genetic apply objects.
static APPLY_TOP_SCHEMA: Lazy<JSONSchema> = Lazy::new(|| {
    let schema = include_str!("apply.schema.json");
    let schema = serde_json::from_str(schema).expect("invalid JSON schema for APPLY_TOP_SCHEMA");
    JSONSchema::compile(&schema).expect("invalid JSON schema for APPLY_TOP_SCHEMA")
});

/// Handle requests to validate API objects.
#[actix_web::post("/apply")]
async fn apply(
    context: Context,
    injector: Data<Injector>,
    object: Json<serde_json::Value>,
) -> Result<HttpResponse, super::Error> {
    // Validate the payload to ensure it follows the required apply format.
    APPLY_TOP_SCHEMA
        .validate(&object)
        .map_err(crate::api::format_json_schema_errors)?;
    let args = ApplyArgs {
        context,
        injector,
        object: &object,
    };

    // Lookup object specific logic based on apiVersion and kind.
    // Unwrapping is safe because object passed validation at this point.
    let kind = object.get("kind").unwrap().as_str().unwrap().to_lowercase();
    let api_version = object.get("apiVersion").unwrap().as_str().unwrap();
    if api_version == self::v0::constants::API_VERSION && self::v0::knows(&kind) {
        return self::v0::apply(args).await;
    }

    // Reject apply request for all other object versions and kinds.
    let response = HttpResponse::BadRequest().json(serde_json::json!({
        "error": true,
        "error_msg": "resource not supported",
        "resource": {
            "apiVersion": api_version,
            "kind": kind,
        },
    }));
    Ok(response)
}
