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

pub struct Meta {
    data: web::Data<MetaData>,
    logger: Logger,
    tracer: Arc<opentracingrust::Tracer>,
}

impl Meta {
    pub fn new(interfaces: &mut Interfaces) -> Self {
        let data = MetaData {
            store: interfaces.stores.primary.clone(),
        };
        Meta {
            data: web::Data::new(data),
            logger: interfaces.logger.clone(),
            tracer: interfaces.tracing.tracer(),
        }
    }

    pub fn resource(&self) -> impl HttpServiceFactory {
        let logger = self.logger.clone();
        let tracer = Arc::clone(&self.tracer);
        let tracer = TracingMiddleware::with_name(logger, tracer, "/cluster/{cluster_id}/meta");
        web::resource("/meta")
            .app_data(self.data.clone())
            .wrap(tracer)
            .route(web::get().to(responder))
    }
}

#[derive(Clone)]
struct MetaData {
    store: Store,
}

async fn responder(data: web::Data<MetaData>, request: HttpRequest) -> Result<impl Responder> {
    let path = request.match_info();
    let cluster_id = path
        .get("cluster_id")
        .ok_or(ErrorKind::APIRequestParameterNotFound("cluster_id"))?
        .to_string();
    let mut request = request;
    let meta = with_request_span(&mut request, |span| -> Result<_> {
        let span = span.map(|span| span.context().clone());
        let meta = data
            .store
            .legacy()
            .cluster_meta(cluster_id.clone(), span)
            .with_context(|_| ErrorKind::PrimaryStoreQuery("cluster_meta"))?
            .ok_or(ErrorKind::ModelNotFound("cluster_meta", cluster_id))?;
        Ok(meta)
    })?;
    let response = HttpResponse::Ok().json(meta);
    Ok(response)
}
