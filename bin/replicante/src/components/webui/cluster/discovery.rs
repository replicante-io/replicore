use std::sync::Arc;

use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use failure::ResultExt;
use slog::Logger;

use replicante_store_primary::store::Store;
use replicante_util_actixweb::with_request_span;
use replicante_util_actixweb::TracingMiddleware;

use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

pub struct Discovery {
    data: DiscoveryData,
    logger: Logger,
    tracer: Arc<opentracingrust::Tracer>,
}

impl Discovery {
    pub fn new(interfaces: &mut Interfaces) -> Discovery {
        let data = DiscoveryData {
            store: interfaces.stores.primary.clone(),
        };
        Discovery {
            data,
            logger: interfaces.logger.clone(),
            tracer: interfaces.tracing.tracer(),
        }
    }

    pub fn resource(&self) -> impl HttpServiceFactory {
        let logger = self.logger.clone();
        let tracer = Arc::clone(&self.tracer);
        let tracer =
            TracingMiddleware::with_name(logger, tracer, "/cluster/{cluster_id}/discovery");
        web::resource("/discovery")
            .data(self.data.clone())
            .wrap(tracer)
            .route(web::get().to(responder))
    }
}

#[derive(Clone)]
struct DiscoveryData {
    store: Store,
}

async fn responder(data: web::Data<DiscoveryData>, request: HttpRequest) -> Result<impl Responder> {
    let path = request.match_info();
    let cluster_id = path
        .get("cluster_id")
        .ok_or_else(|| ErrorKind::APIRequestParameterNotFound("cluster_id"))?
        .to_string();

    let mut request = request;
    let discovery = with_request_span(&mut request, |span| -> Result<_> {
        let span = span.map(|span| span.context().clone());
        let discovery = data
            .store
            .cluster(cluster_id.clone())
            .discovery(span)
            .with_context(|_| ErrorKind::PrimaryStoreQuery("cluster.discovery"))?
            .ok_or_else(|| ErrorKind::ModelNotFound("ClusterDiscovery", cluster_id))?;
        Ok(discovery)
    })?;
    let response = HttpResponse::Ok().json(discovery);
    Ok(response)
}
