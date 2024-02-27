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
        let tracer =
            TracingMiddleware::with_name(logger, tracer, "/platform/{namespace}/{platform}");
        web::resource("/{platform}")
            .app_data(self.data.clone())
            .wrap(tracer)
            .route(web::get().to(responder))
    }
}

async fn responder(data: web::Data<GetData>, request: HttpRequest) -> Result<impl Responder> {
    let path = request.match_info();
    let ns_id = path
        .get("namespace")
        .ok_or(ErrorKind::APIRequestParameterNotFound("namespace"))?
        .to_string();
    let platform_id = path
        .get("platform")
        .ok_or(ErrorKind::APIRequestParameterNotFound("platform"))?
        .to_string();

    let mut request = request;
    let platform = with_request_span(&mut request, |span| {
        let span = span.map(|span| span.context().clone());
        data.store
            .platform(ns_id, platform_id)
            .get(span)
            .with_context(|_| ErrorKind::PrimaryStoreQuery("platform"))
    })?;

    let response = match platform {
        Some(platform) => HttpResponse::Ok().json(platform),
        None => HttpResponse::NotFound().json(serde_json::json!({
            "error": "platform not found",
            "layers": [],
        })),
    };
    Ok(response)
}

#[derive(Clone)]
struct GetData {
    store: Store,
}
