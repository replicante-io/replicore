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

use crate::interfaces::Interfaces;
use crate::ErrorKind;
use crate::Result;

pub struct List {
    data: web::Data<ListData>,
    logger: Logger,
    tracer: Arc<opentracingrust::Tracer>,
}

impl List {
    pub fn new(logger: &Logger, interfaces: &mut Interfaces) -> List {
        let data = ListData {
            store: interfaces.stores.primary.clone(),
        };
        List {
            data: web::Data::new(data),
            logger: logger.clone(),
            tracer: interfaces.tracing.tracer(),
        }
    }

    pub fn resource(&self) -> impl HttpServiceFactory {
        let logger = self.logger.clone();
        let tracer = Arc::clone(&self.tracer);
        let tracer = TracingMiddleware::with_name(logger, tracer, "/platforms/{namespace}");
        web::resource("")
            .app_data(self.data.clone())
            .wrap(tracer)
            .route(web::get().to(responder))
    }
}

async fn responder(data: web::Data<ListData>, request: HttpRequest) -> Result<impl Responder> {
    let path = request.match_info();
    let ns_id = path
        .get("namespace")
        .ok_or(ErrorKind::APIRequestParameterNotFound("namespace"))?
        .to_string();

    let mut request = request;
    let cursor = with_request_span(&mut request, |span| {
        let span = span.map(|span| span.context().clone());
        data.store
            .platforms(ns_id)
            .iter(span)
            .with_context(|_| ErrorKind::PrimaryStoreQuery("platforms"))
    })?;

    let mut platforms = vec![];
    for platform in cursor {
        let platform = platform.with_context(|_| ErrorKind::PrimaryStoreQuery("Platform"))?;
        platforms.push(platform);
    }

    let response = HttpResponse::Ok().json(platforms);
    Ok(response)
}

#[derive(Clone)]
struct ListData {
    store: Store,
}
