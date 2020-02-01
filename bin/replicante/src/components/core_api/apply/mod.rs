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

use replicante_store_primary::store::Store as PrimaryStore;
use replicante_store_view::store::Store as ViewStore;
use replicante_util_iron::request_span;

use crate::interfaces::api::APIRoot;
use crate::interfaces::Interfaces;
use crate::Error;
use crate::ErrorKind;

mod agent_action;
mod appliers;
mod metrics;
mod validate;

pub use metrics::register_metrics;
use metrics::APPLY_COUNT;
use metrics::APPLY_DURATION;
use metrics::APPLY_ERROR;
use metrics::APPLY_UNKNOWN;

/// Attach the endpoint to handle all `apply` requests.
pub fn attach(logger: Logger, interfaces: &mut Interfaces) {
    let mut router = interfaces.api.router_for(&APIRoot::UnstableCoreApi);
    let handler = Apply {
        logger,
        primary_store: interfaces.stores.primary.clone(),
        view_store: interfaces.stores.view.clone(),
    };
    router.post("/apply", handler, "/apply");
}

/// Endpoint handling all requests for system changes (apply requests).
pub struct Apply {
    logger: Logger,
    primary_store: PrimaryStore,
    view_store: ViewStore,
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
        let api_version = object.api_version.clone();
        let kind = object.kind.clone();
        APPLY_COUNT.with_label_values(&[&api_version, &kind]).inc();
        let applier = match appliers::find(&object) {
            Some(applier) => applier,
            None => {
                APPLY_UNKNOWN
                    .with_label_values(&[&api_version, &kind])
                    .inc();
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
        let timer = APPLY_DURATION
            .with_label_values(&[&api_version, &kind])
            .start_timer();
        let result = applier(appliers::ApplierArgs {
            object,
            primary_store: self.primary_store.clone(),
            span: Some(request_span(request)),
            view_store: self.view_store.clone(),
        });
        timer.observe_duration();
        let mut result = match result {
            Ok(result) => result,
            Err(error) => {
                APPLY_ERROR.with_label_values(&[&api_version, &kind]).inc();
                return Err(error.into());
            }
        };
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
