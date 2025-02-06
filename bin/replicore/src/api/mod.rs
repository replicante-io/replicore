//! API related tools (such as middlewares) and endpoints.
use actix_web::web::Data;
use actix_web::web::ServiceConfig;
use actix_web::HttpResponse;
use jsonschema::Validator;
use serde_json::Value;

use replisdk::utils::actix::error::Error;

use replicore_injector::Injector;

pub mod apply;
pub mod constants;
pub mod context;
pub mod lease;
pub mod object;

/// Successful (200) API response with no data returned to the client.
#[inline]
pub fn done() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({}))
}

/// Not Found (404) API response, commonly for non-existing records.
#[inline]
pub fn not_found() -> HttpResponse {
    HttpResponse::NotFound().json(serde_json::json!({}))
}

/// Configure an HTTP Server with all endpoints in this API module.
pub fn configure(config: &mut ServiceConfig) {
    let injector = Injector::global();
    let scope = actix_web::web::scope("/api/v0")
        .app_data(Data::new(injector))
        .service(self::apply::apply)
        .configure(self::object::configure);
    config.service(scope);
}

/// Format `jsonschema` validation errors into an [`actix_web`] compatible response.
///
/// [`jsonschema`]: jsonschema::Validator
fn validate_schema(validator: &Validator, object: &Value) -> Result<(), Error> {
    let errors = validator.iter_errors(object);
    let mut violations = Vec::new();
    for error in errors {
        violations.push(error.to_string());
    }

    if violations.is_empty() {
        return Ok(());
    }

    let json = serde_json::json!({
        "error": true,
        "error_msg": "Payload validation failed",
        "violations": violations,
    });
    let source = anyhow::anyhow!("JSON Schema validation of request payload failed");
    Err(Error::bad_request(source).use_strategy(json))
}
