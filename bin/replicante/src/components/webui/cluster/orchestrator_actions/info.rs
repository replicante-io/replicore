use std::sync::Arc;

use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use failure::ResultExt;
use serde_json::json;
use slog::Logger;
use uuid::Uuid;

use replicante_store_view::store::Store;
use replicante_util_actixweb::with_request_span;
use replicante_util_actixweb::TracingMiddleware;

use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

pub struct OrchestratorActionInfo {
    data: OrchestratorActionData,
    logger: Logger,
    tracer: Arc<opentracingrust::Tracer>,
}

impl OrchestratorActionInfo {
    pub fn new(interfaces: &mut Interfaces) -> OrchestratorActionInfo {
        let data = OrchestratorActionData {
            store: interfaces.stores.view.clone(),
        };
        OrchestratorActionInfo {
            data,
            logger: interfaces.logger.clone(),
            tracer: interfaces.tracing.tracer(),
        }
    }

    pub fn resource(&self) -> impl HttpServiceFactory {
        let logger = self.logger.clone();
        let tracer = Arc::clone(&self.tracer);
        let tracer = TracingMiddleware::with_name(
            logger,
            tracer,
            "/cluster/{cluster_id}/orchestrator-action/{action_id}",
        );
        web::resource("/orchestrator-action/{action_id}")
            .data(self.data.clone())
            .wrap(tracer)
            .route(web::get().to(responder))
    }
}

async fn responder(
    data: web::Data<OrchestratorActionData>,
    request: HttpRequest,
) -> Result<impl Responder> {
    let path = request.match_info();
    let cluster_id = path
        .get("cluster_id")
        .ok_or(ErrorKind::APIRequestParameterNotFound("cluster_id"))?
        .to_string();
    let action_id = path
        .get("action_id")
        .ok_or(ErrorKind::APIRequestParameterNotFound("action_id"))?;
    let action_id = Uuid::parse_str(action_id)
        .with_context(|_| ErrorKind::APIRequestParameterInvalid("action_id"))?;

    // Fetch requested action information.
    let mut request = request;
    let store = data.store.orchestrator_actions(cluster_id);
    let action = with_request_span(&mut request, |span| {
        let span = span.map(|span| span.context().clone());
        store
            .orchestrator_action(action_id, span)
            .with_context(|_| ErrorKind::ViewStoreQuery("orchestrator action"))
    })?;
    let action = match action {
        Some(action) => action,
        None => {
            let response = HttpResponse::NotFound().json(json!({}));
            return Ok(response);
        }
    };

    // Send the response.
    let response = HttpResponse::Ok().json(action);
    Ok(response)
}

#[derive(Clone)]
struct OrchestratorActionData {
    store: Store,
}
