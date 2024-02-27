use std::sync::Arc;

use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use failure::ResultExt;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use slog::Logger;
use uuid::Uuid;

use replicante_models_core::actions::node::Action;
use replicante_models_core::actions::node::ActionHistory;
use replicante_store_view::store::Store;
use replicante_util_actixweb::with_request_span;
use replicante_util_actixweb::TracingMiddleware;

use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

pub struct ActionInfo {
    data: web::Data<ActionData>,
    logger: Logger,
    tracer: Arc<opentracingrust::Tracer>,
}

impl ActionInfo {
    pub fn new(interfaces: &mut Interfaces) -> ActionInfo {
        let data = ActionData {
            store: interfaces.stores.view.clone(),
        };
        ActionInfo {
            data: web::Data::new(data),
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
            "/cluster/{cluster_id}/action/{action_id}",
        );
        web::resource("/action/{action_id}")
            .app_data(self.data.clone())
            .wrap(tracer)
            .route(web::get().to(responder))
    }
}

async fn responder(data: web::Data<ActionData>, request: HttpRequest) -> Result<impl Responder> {
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
    let store = data.store.actions(cluster_id);
    let action = with_request_span(&mut request, |span| {
        let span = span.map(|span| span.context().clone());
        store
            .action(action_id, span)
            .with_context(|_| ErrorKind::ViewStoreQuery("action"))
    })?;
    let action = match action {
        Some(action) => action,
        None => {
            let response = HttpResponse::NotFound().json(json!({}));
            return Ok(response);
        }
    };

    let history = with_request_span(&mut request, |span| {
        let span = span.map(|span| span.context().clone());
        store
            .history(action_id, span)
            .with_context(|_| ErrorKind::ViewStoreQuery("action history"))
    })?;

    // Send the response.
    let info = ActionResponse { action, history };
    let response = HttpResponse::Ok().json(info);
    Ok(response)
}

#[derive(Clone)]
struct ActionData {
    store: Store,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
struct ActionResponse {
    pub action: Action,
    pub history: Vec<ActionHistory>,
}
