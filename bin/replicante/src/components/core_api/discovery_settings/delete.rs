use std::sync::Arc;

use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use failure::ResultExt;
use serde_json::json;
use slog::Logger;

use replicante_store_primary::store::Store;
use replicante_util_actixweb::with_request_span;
use replicante_util_actixweb::TracingMiddleware;

use crate::interfaces::Interfaces;
use crate::ErrorKind;
use crate::Result;

pub struct Delete {
    data: DeleteData,
    logger: Logger,
    tracer: Arc<opentracingrust::Tracer>,
}

impl Delete {
    pub fn new(logger: &Logger, interfaces: &mut Interfaces) -> Delete {
        let data = DeleteData {
            store: interfaces.stores.primary.clone(),
        };
        Delete {
            data,
            logger: logger.clone(),
            tracer: interfaces.tracing.tracer(),
        }
    }

    pub fn resource(&self) -> impl HttpServiceFactory {
        let logger = self.logger.clone();
        let tracer = Arc::clone(&self.tracer);
        let tracer = TracingMiddleware::with_name(
            logger,
            tracer,
            "/discoverysettings/{namespace}/{name}/delete",
        );
        web::resource("/{name}/delete")
            .data(self.data.clone())
            .wrap(tracer)
            .route(web::delete().to(responder))
    }
}

async fn responder(data: web::Data<DeleteData>, request: HttpRequest) -> Result<impl Responder> {
    let path = request.match_info();
    let namespace = path
        .get("namespace")
        .ok_or_else(|| ErrorKind::APIRequestParameterNotFound("namespace"))?
        .to_string();
    let name = path
        .get("name")
        .ok_or_else(|| ErrorKind::APIRequestParameterNotFound("name"))?
        .to_string();

    let mut request = request;
    with_request_span(&mut request, |span| {
        let span = span.map(|span| span.context().clone());
        data.store
            .discovery_settings(namespace)
            .delete(&name, span)
            .with_context(|_| ErrorKind::PrimaryStoreDelete("discovery settings"))
    })?;

    let response = HttpResponse::Ok().json(json!({}));
    Ok(response)
}

#[derive(Clone)]
struct DeleteData {
    store: Store,
}
