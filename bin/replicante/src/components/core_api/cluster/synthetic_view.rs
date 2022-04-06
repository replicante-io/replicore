use std::sync::Arc;

use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use failure::ResultExt;
use slog::Logger;

use replicante_models_core::scope::Namespace;
use replicante_store_primary::store::Store;
use replicante_util_actixweb::with_request_span;
use replicante_util_actixweb::TracingMiddleware;

use crate::interfaces::Interfaces;
use crate::ErrorKind;
use crate::Result;

pub struct SyntheticView {
    data: SyntheticViewData,
    tracer: Arc<opentracingrust::Tracer>,
}

impl SyntheticView {
    pub fn new(logger: &Logger, interfaces: &mut Interfaces) -> SyntheticView {
        let data = SyntheticViewData {
            logger: logger.clone(),
            store: interfaces.stores.primary.clone(),
        };
        SyntheticView {
            data,
            tracer: interfaces.tracing.tracer(),
        }
    }

    pub fn resource(&self) -> impl HttpServiceFactory {
        let logger = self.data.logger.clone();
        let tracer = Arc::clone(&self.tracer);
        let tracer =
            TracingMiddleware::with_name(logger, tracer, "/cluster/{cluster_id}/synthetic_view");
        web::resource("/synthetic_view")
            .data(self.data.clone())
            .wrap(tracer)
            .route(web::get().to(responder))
    }
}

#[derive(Clone)]
struct SyntheticViewData {
    logger: Logger,
    store: Store,
}

async fn responder(
    data: web::Data<SyntheticViewData>,
    request: HttpRequest,
) -> Result<impl Responder> {
    let path = request.match_info();
    let cluster_id = path
        .get("cluster_id")
        .ok_or(ErrorKind::APIRequestParameterNotFound("cluster_id"))?
        .to_string();

    // Build and fetch the cluster view.
    let mut request = request;
    let namespace = Namespace::HARDCODED_FOR_ROLLOUT().ns_id;
    let view = with_request_span(&mut request, |span| -> Result<_> {
        let span = span.map(|span| span.context().clone());
        let view = data
            .store
            .cluster_view(namespace, cluster_id, span)
            .with_context(|_| ErrorKind::PrimaryStoreQuery("cluster_view"))?;
        Ok(view)
    })?;

    // Return serialised view.
    let response = HttpResponse::Ok().json(view);
    Ok(response)
}
