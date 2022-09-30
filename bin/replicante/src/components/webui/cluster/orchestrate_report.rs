use std::sync::Arc;

use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use failure::ResultExt;
use slog::Logger;

use replicante_models_core::scope::Namespace;
use replicante_store_view::store::Store;
use replicante_util_actixweb::with_request_span;
use replicante_util_actixweb::TracingMiddleware;

use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

pub struct OrchestrateReport {
    data: web::Data<OrchestrateReportData>,
    logger: Logger,
    tracer: Arc<opentracingrust::Tracer>,
}

impl OrchestrateReport {
    pub fn new(interfaces: &mut Interfaces) -> OrchestrateReport {
        let data = OrchestrateReportData {
            store: interfaces.stores.view.clone(),
        };
        OrchestrateReport {
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
            "/cluster/{cluster_id}/orchestrate_report",
        );
        web::resource("/orchestrate_report")
            .app_data(self.data.clone())
            .wrap(tracer)
            .route(web::get().to(responder))
    }
}

#[derive(Clone)]
struct OrchestrateReportData {
    store: Store,
}

async fn responder(
    data: web::Data<OrchestrateReportData>,
    request: HttpRequest,
) -> Result<impl Responder> {
    let path = request.match_info();
    let cluster_id = path
        .get("cluster_id")
        .ok_or(ErrorKind::APIRequestParameterNotFound("cluster_id"))?
        .to_string();

    let mut request = request;
    let namespace = Namespace::HARDCODED_FOR_ROLLOUT();
    let report = with_request_span(&mut request, |span| -> Result<_> {
        let span = span.map(|span| span.context().clone());
        let report = data
            .store
            .cluster(&namespace.ns_id, &cluster_id)
            .orchestrate_report(span)
            .with_context(|_| ErrorKind::PrimaryStoreQuery("cluster.orchestrate_report"))?
            .ok_or(ErrorKind::ModelNotFound("OrchestrateReport", cluster_id))?;
        Ok(report)
    })?;
    let response = HttpResponse::Ok().json(report);
    Ok(response)
}
