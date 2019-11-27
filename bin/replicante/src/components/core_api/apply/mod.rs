use failure::ResultExt;
use iron::headers::ContentType;
use iron::status;
use iron::Handler;
use iron::IronError;
use iron::IronResult;
use iron::Plugin;
use iron::Request;
use iron::Response;
use iron::Set;
use iron_json_response::JsonResponse;
use serde_json::json;
use slog::debug;
use slog::Logger;

use crate::interfaces::api::APIRoot;
use crate::interfaces::Interfaces;
use crate::Error;
use crate::ErrorKind;

mod agent_action;
mod appliers;
mod validate;

/// Attach the endpoint to handle all `apply` requests.
pub fn attach(logger: Logger, interfaces: &mut Interfaces) {
    let mut router = interfaces.api.router_for(&APIRoot::UnstableCoreApi);
    let handler = Apply { logger };
    router.post("/apply", handler, "/apply");
}

/// Endpoint handling all requests for system changes (apply requests).
pub struct Apply {
    logger: Logger,
}

impl Handler for Apply {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        // Get apply object from the request.
        debug!(self.logger, "Handling apply request");
        let object = request
            .get::<bodyparser::Json>()
            .with_context(|_| ErrorKind::APIRequestBodyInvalid)
            .map_err(Error::from)?
            .ok_or_else(|| ErrorKind::APIRequestBodyNotFound)
            .map_err(Error::from)?;

        // Validate basic attributes and find an "applier" for it.
        let object = validate::required_attributes(object)?;
        let applier = match appliers::find(&object) {
            Some(applier) => applier,
            None => {
                let msg = format!(
                    "{} objects are not supported by {}",
                    object.kind, object.api_version
                );
                let payload = json!({
                    "apiVersion": object.api_version,
                    "kind": object.kind,
                    "error": &msg,
                    "layers": vec![msg],
                });
                let mut response = Response::with((
                    status::MethodNotAllowed,
                    serde_json::to_string(&payload).unwrap(),
                ));
                response.headers.set(ContentType::json());
                let error = failure::err_msg("unsupported kind for apiVersion").compat();
                return Err(IronError {
                    error: Box::new(error),
                    response,
                });
            }
        };

        // Handle the apply request.
        // The applier is expected to do any version & kind validation.
        let mut result = applier(appliers::ApplierArgs { object })?;
        let response = match result {
            _ if result.is_null() => json!({"ok": 1}),
            _ if result.is_object() => {
                result
                    .as_object_mut()
                    .expect("serde_json::Value::is_object lied")
                    .insert("ok".to_string(), 1.into());
                result
            }
            _ => json!({
                "ok": 1,
                "response": result,
            }),
        };

        // Ack the request.
        let mut resp = Response::new();
        resp.set_mut(JsonResponse::json(response))
            .set_mut(status::Ok);
        Ok(resp)
    }
}
