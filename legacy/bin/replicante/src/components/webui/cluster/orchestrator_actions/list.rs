use std::sync::Arc;

use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use chrono::DateTime;
use chrono::Utc;
use failure::ResultExt;
use serde::Deserialize;
use serde::Serialize;
use slog::Logger;

use replicante_models_core::actions::orchestrator::OrchestratorAction;
use replicante_store_view::store::orchestrator_actions::SearchFilters;
use replicante_store_view::store::Store;
use replicante_util_actixweb::with_request_span;
use replicante_util_actixweb::TracingMiddleware;

use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

pub struct OrchestratorActions {
    data: web::Data<OrchestratorActionsData>,
    logger: Logger,
    tracer: Arc<opentracingrust::Tracer>,
}

impl OrchestratorActions {
    pub fn new(interfaces: &mut Interfaces) -> OrchestratorActions {
        let data = OrchestratorActionsData {
            store: interfaces.stores.view.clone(),
        };
        OrchestratorActions {
            data: web::Data::new(data),
            logger: interfaces.logger.clone(),
            tracer: interfaces.tracing.tracer(),
        }
    }

    pub fn resource(&self) -> impl HttpServiceFactory {
        let logger = self.logger.clone();
        let tracer = Arc::clone(&self.tracer);
        let tracer =
            TracingMiddleware::with_name(logger, tracer, "/cluster/{cluster}/orchestrator-actions");
        web::resource("/orchestrator-actions")
            .app_data(self.data.clone())
            .wrap(tracer)
            .route(web::post().to(responder))
    }
}

async fn responder(
    filters: web::Json<OrchestratorActionsSearch>,
    data: web::Data<OrchestratorActionsData>,
    request: HttpRequest,
) -> Result<impl Responder> {
    let path = request.match_info();
    let cluster_id = path
        .get("cluster_id")
        .ok_or(ErrorKind::APIRequestParameterNotFound("cluster_id"))?
        .to_string();

    // Search for actions and prepare results.
    let mut request = request;
    let filters = SearchFilters::from(filters.into_inner());
    let cursor = with_request_span(&mut request, |span| {
        let span = span.map(|span| span.context().clone());
        data.store
            .orchestrator_actions(cluster_id)
            .search(filters, span)
            .with_context(|_| ErrorKind::ViewStoreQuery("orchestrator actions"))
    })?;

    let mut actions: Vec<OrchestratorAction> = Vec::new();
    for action in cursor {
        let action = action.with_context(|_| {
            ErrorKind::Deserialize("orchestrator action record", "OrchestratorAction")
        })?;
        actions.push(action);
    }

    // Send the response.
    let response = HttpResponse::Ok().json(actions);
    Ok(response)
}

#[derive(Clone)]
struct OrchestratorActionsData {
    store: Store,
}

/// Filters to search actions to be returned.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
struct OrchestratorActionsSearch {
    action_kind: String,
    action_state: String,
    from: DateTime<Utc>,
    until: DateTime<Utc>,
}

impl From<OrchestratorActionsSearch> for SearchFilters {
    fn from(filters: OrchestratorActionsSearch) -> SearchFilters {
        let action_kind = if filters.action_kind.is_empty() {
            None
        } else {
            Some(filters.action_kind)
        };
        let action_state = if filters.action_state.is_empty() {
            None
        } else {
            Some(filters.action_state)
        };
        SearchFilters {
            action_kind,
            action_state,
            from: filters.from,
            until: filters.until,
        }
    }
}
