//! API related tools (such as middlewares) and endpoints.
use actix_web::web::Data;
use actix_web::web::ServiceConfig;
use jsonschema::ErrorIterator;

use replisdk::utils::actix::error::Error;

use replicore_injector::Injector;

pub mod apply;
pub mod context;
// TODO: pub mod list;
// TODO: pub mod object;

/// Configure an HTTP Server with all endpoints in this API module.
pub fn configure(config: &mut ServiceConfig) {
    let injector = Injector::global();
    let scope = actix_web::web::scope("/api/v0")
        .app_data(Data::new(injector))
        .service(self::apply::apply);
    config.service(scope);
}

/// Format [`JSONSchema`] validation errors into an [actix_web] compatible response.
///
/// [`JSONSchema`]: jsonschema::JSONSchema
fn format_json_schema_errors(errors: ErrorIterator) -> Error {
    let mut violations = Vec::new();
    for error in errors {
        violations.push(error.to_string());
    }
    let json = serde_json::json!({
        "error": true,
        "error_msg": "Payload validation failed",
        "violations": violations,
    });
    let source = anyhow::anyhow!("JSON Schema validation of request payload failed");
    Error::bad_request(source).use_strategy(json)
}
