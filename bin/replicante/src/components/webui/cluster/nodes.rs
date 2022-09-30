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

pub struct Nodes {
    data: web::Data<NodesData>,
    logger: Logger,
    tracer: Arc<opentracingrust::Tracer>,
}

impl Nodes {
    pub fn new(interfaces: &mut Interfaces) -> Self {
        let data = NodesData {
            store: interfaces.stores.primary.clone(),
        };
        Nodes {
            data: web::Data::new(data),
            logger: interfaces.logger.clone(),
            tracer: interfaces.tracing.tracer(),
        }
    }

    pub fn resource(&self) -> impl HttpServiceFactory {
        let logger = self.logger.clone();
        let tracer = Arc::clone(&self.tracer);
        let tracer = TracingMiddleware::with_name(logger, tracer, "/cluster/{cluster_id}/nodes");
        web::resource("/nodes")
            .app_data(self.data.clone())
            .wrap(tracer)
            .route(web::get().to(responder))
    }
}

#[derive(Clone)]
struct NodesData {
    store: Store,
}

async fn responder(data: web::Data<NodesData>, request: HttpRequest) -> Result<impl Responder> {
    let path = request.match_info();
    let cluster_id = path
        .get("cluster_id")
        .ok_or(ErrorKind::APIRequestParameterNotFound("cluster_id"))?
        .to_string();

    let mut nodes = Vec::new();
    let mut request = request;
    let iter = with_request_span(&mut request, |span| {
        let span = span.map(|span| span.context().clone());
        data.store
            .nodes(cluster_id)
            .iter(span)
            .with_context(|_| ErrorKind::PrimaryStoreQuery("nodes.iter"))
    })?;
    for node in iter {
        let node = node.with_context(|_| ErrorKind::Deserialize("node record", "Node"))?;
        nodes.push(node);
    }

    let response = HttpResponse::Ok().json(nodes);
    Ok(response)
}
