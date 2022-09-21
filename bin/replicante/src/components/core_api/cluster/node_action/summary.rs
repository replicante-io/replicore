use std::sync::Arc;

use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use failure::ResultExt;
use slog::Logger;

use replicante_models_core::api::node_action::NodeActionSummariesResponse;
use replicante_store_primary::store::Store;
use replicante_util_actixweb::with_request_span;
use replicante_util_actixweb::TracingMiddleware;

use crate::interfaces::Interfaces;
use crate::ErrorKind;
use crate::Result;

pub struct Summary {
    data: SummaryData,
    tracer: Arc<opentracingrust::Tracer>,
}

impl Summary {
    pub fn new(logger: &Logger, interfaces: &mut Interfaces) -> Summary {
        let data = SummaryData {
            logger: logger.clone(),
            store: interfaces.stores.primary.clone(),
        };
        Summary {
            data,
            tracer: interfaces.tracing.tracer(),
        }
    }

    pub fn resource(&self) -> impl HttpServiceFactory {
        let logger = self.data.logger.clone();
        let tracer = Arc::clone(&self.tracer);
        let tracer =
            TracingMiddleware::with_name(logger, tracer, "/cluster/{cluster_id}/action/summary");
        web::resource("/action/summary")
            .data(self.data.clone())
            .wrap(tracer)
            .route(web::get().to(responder))
    }
}

async fn responder(data: web::Data<SummaryData>, request: HttpRequest) -> Result<impl Responder> {
    let path = request.match_info();
    let cluster_id = path
        .get("cluster_id")
        .ok_or(ErrorKind::APIRequestParameterNotFound("cluster_id"))?
        .to_string();

    let mut request = request;
    let cursor = with_request_span(&mut request, |span| {
        let span = span.map(|span| span.context().clone());
        data.store
            .actions(cluster_id)
            .iter_summary(span)
            .with_context(|_| ErrorKind::PrimaryStoreQuery("action summary"))
    })?;

    let mut actions = vec![];
    for summary in cursor {
        let summary = summary.with_context(|_| ErrorKind::PrimaryStoreQuery("action summary"))?;
        actions.push(summary);
    }

    let response = NodeActionSummariesResponse { actions };
    let response = HttpResponse::Ok().json(response);
    Ok(response)
}

#[derive(Clone)]
struct SummaryData {
    logger: Logger,
    store: Store,
}
