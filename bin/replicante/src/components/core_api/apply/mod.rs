use std::collections::HashSet;
use std::sync::Arc;

use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use serde_json::json;
use serde_json::Value;
use slog::debug;
use slog::Logger;

use replicante_store_primary::store::Store as PrimaryStore;
use replicante_store_view::store::Store as ViewStore;
use replicante_util_actixweb::with_request_span;
use replicante_util_actixweb::RootDescriptor;
use replicante_util_actixweb::TracingMiddleware;

use crate::interfaces::api::APIRoot;
use crate::interfaces::api::AppConfigContext;
use crate::interfaces::Interfaces;
use crate::Result;

mod agent_action;
mod appliers;
mod metrics;
mod validate;

pub use metrics::register_metrics;
use metrics::APPLY_COUNT;
use metrics::APPLY_DURATION;
use metrics::APPLY_ERROR;
use metrics::APPLY_UNKNOWN;

lazy_static::lazy_static! {
    /// Set of HTTP headers to exclude when collecting action headers.
    static ref HTTP_HEADERS_IGNORE: HashSet<String> = {
        let mut headers = HashSet::new();
        headers.insert("accept".into());
        headers.insert("accept-encoding".into());
        headers.insert("content-length".into());
        headers.insert("content-type".into());
        headers.insert("host".into());
        headers.insert("user-agent".into());
        headers
    };
}

/// Return an `AppConfig` callback to configure the apply endpoint.
pub fn configure(logger: &Logger, interfaces: &mut Interfaces) -> impl Fn(&mut AppConfigContext) {
    let apply = ApplyData {
        logger: logger.clone(),
        primary_store: interfaces.stores.primary.clone(),
        view_store: interfaces.stores.view.clone(),
    };
    let apply = Apply {
        data: apply,
        tracer: interfaces.tracing.tracer(),
    };
    move |conf| {
        APIRoot::UnstableCoreApi.and_then(&conf.context.flags, |root| {
            conf.scoped_service(root.prefix(), apply.resource());
        });
    }
}

/// Endpoint handling all requests for system changes (apply requests).
pub struct Apply {
    data: ApplyData,
    tracer: Arc<opentracingrust::Tracer>,
}

impl Apply {
    pub fn resource(&self) -> impl HttpServiceFactory {
        let logger = self.data.logger.clone();
        let tracer = Arc::clone(&self.tracer);
        web::resource("/apply")
            .data(self.data.clone())
            .wrap(TracingMiddleware::new(logger, tracer))
            .route(web::post().to(responder))
    }
}

#[derive(Clone)]
struct ApplyData {
    logger: Logger,
    primary_store: PrimaryStore,
    view_store: ViewStore,
}

async fn responder(
    object: web::Json<Value>,
    data: web::Data<ApplyData>,
    request: HttpRequest,
) -> Result<impl Responder> {
    debug!(data.logger, "Handling apply request");
    let object = object.into_inner();
    let mut request = request;

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
            let response = HttpResponse::MethodNotAllowed().json(payload);
            return Ok(response);
        }
    };

    // Handle the apply request.
    // The applier is expected to do any version & kind validation.
    let headers = request
        .headers()
        .iter()
        .map(|(name, value)| {
            let name = name.as_str().to_string();
            let value = match value.to_str() {
                Ok(value) => value.to_string(),
                Err(_) => "<binary-header-value>".to_string(),
            };
            (name, value)
        })
        .filter(|(name, _)| !HTTP_HEADERS_IGNORE.contains(name))
        .collect();
    let timer = APPLY_DURATION
        .with_label_values(&[&api_version, &kind])
        .start_timer();
    let result = with_request_span(&mut request, |span| {
        applier(appliers::ApplierArgs {
            headers,
            object,
            primary_store: data.primary_store.clone(),
            span,
            view_store: data.view_store.clone(),
        })
    });
    timer.observe_duration();
    let mut result = match result {
        Ok(result) => result,
        Err(error) => {
            APPLY_ERROR.with_label_values(&[&api_version, &kind]).inc();
            return Err(error);
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
    let response = HttpResponse::Ok().json(response);
    Ok(response)
}
