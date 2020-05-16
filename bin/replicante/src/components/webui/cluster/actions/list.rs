use std::sync::Arc;

use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use chrono::DateTime;
use chrono::Utc;
use failure::ResultExt;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use slog::Logger;

use replicante_models_core::actions::Action;
use replicante_store_view::store::actions::SearchFilters;
use replicante_store_view::store::Store;
use replicante_util_actixweb::with_request_span;
use replicante_util_actixweb::TracingMiddleware;

use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

pub struct Actions {
    data: ActionsData,
    logger: Logger,
    tracer: Arc<opentracingrust::Tracer>,
}

impl Actions {
    pub fn new(interfaces: &mut Interfaces) -> Actions {
        let data = ActionsData {
            store: interfaces.stores.view.clone(),
        };
        Actions {
            data,
            logger: interfaces.logger.clone(),
            tracer: interfaces.tracing.tracer(),
        }
    }

    pub fn resource(&self) -> impl HttpServiceFactory {
        let logger = self.logger.clone();
        let tracer = Arc::clone(&self.tracer);
        let tracer = TracingMiddleware::with_name(logger, tracer, "/cluster/{cluster}/actions");
        web::resource("/actions")
            .data(self.data.clone())
            .wrap(tracer)
            .route(web::post().to(responder))
    }
}

async fn responder(
    filters: web::Json<ActionsSearch>,
    data: web::Data<ActionsData>,
    request: HttpRequest,
) -> Result<impl Responder> {
    let path = request.match_info();
    let cluster_id = path
        .get("cluster_id")
        .ok_or_else(|| ErrorKind::APIRequestParameterNotFound("cluster_id"))?
        .to_string();

    // Search for actions and prepare results.
    let mut request = request;
    let filters = SearchFilters::from(filters.into_inner());
    let cursor = with_request_span(&mut request, |span| {
        let span = span.map(|span| span.context().clone());
        data.store
            .actions(cluster_id)
            .search(filters, span)
            .with_context(|_| ErrorKind::ViewStoreQuery("actions"))
    })?;

    let mut actions: Vec<Action> = Vec::new();
    for action in cursor {
        let action = action.with_context(|_| ErrorKind::Deserialize("action record", "Action"))?;
        actions.push(action);
    }

    // Send the response.
    let response = HttpResponse::Ok().json(actions);
    Ok(response)
}

#[derive(Clone)]
struct ActionsData {
    store: Store,
}

/// Filters to search actions to be returned.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
struct ActionsSearch {
    action_kind: String,
    action_state: String,
    from: DateTime<Utc>,
    node_id: String,
    until: DateTime<Utc>,
}

impl From<ActionsSearch> for SearchFilters {
    fn from(filters: ActionsSearch) -> SearchFilters {
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
        let node_id = if filters.node_id.is_empty() {
            None
        } else {
            Some(filters.node_id)
        };
        SearchFilters {
            action_kind,
            action_state,
            from: filters.from,
            node_id,
            until: filters.until,
        }
    }
}
