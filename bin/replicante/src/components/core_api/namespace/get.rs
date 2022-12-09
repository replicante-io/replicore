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

pub struct Get {
    data: web::Data<GetData>,
    logger: Logger,
    tracer: Arc<opentracingrust::Tracer>,
}

impl Get {
    pub fn new(logger: &Logger, interfaces: &mut Interfaces) -> Get {
        let data = GetData {
            store: interfaces.stores.primary.clone(),
        };
        Get {
            data: web::Data::new(data),
            logger: logger.clone(),
            tracer: interfaces.tracing.tracer(),
        }
    }

    pub fn resource(&self) -> impl HttpServiceFactory {
        let logger = self.logger.clone();
        let tracer = Arc::clone(&self.tracer);
        let tracer = TracingMiddleware::with_name(logger, tracer, "/namespace/{namespace}");
        web::resource("/{namespace}")
            .app_data(self.data.clone())
            .wrap(tracer)
            .route(web::get().to(responder))
    }
}

async fn responder(data: web::Data<GetData>, request: HttpRequest) -> Result<impl Responder> {
    let path = request.match_info();
    let namespace_id = path
        .get("namespace")
        .ok_or(ErrorKind::APIRequestParameterNotFound("namespace"))?
        .to_string();

    let mut request = request;
    let namespace = with_request_span(&mut request, |span| {
        let span = span.map(|span| span.context().clone());
        data.store
            .namespace(namespace_id)
            .get(span)
            .with_context(|_| ErrorKind::PrimaryStoreQuery("namespace"))
    })?;

    let response = match namespace {
        Some(namespace) => HttpResponse::Ok().json(namespace),
        None => HttpResponse::NotFound().json(serde_json::json!({
            "error": "namespace not found",
            "layers": [],
        })),
    };
    Ok(response)
}

#[derive(Clone)]
struct GetData {
    store: Store,
}
