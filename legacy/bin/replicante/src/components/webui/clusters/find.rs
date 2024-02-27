use std::sync::Arc;

use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Resource;
use actix_web::Responder;
use failure::ResultExt;
use slog::Logger;

use replicante_store_primary::store::Store;
use replicante_util_actixweb::with_request_span;
use replicante_util_actixweb::TracingMiddleware;

use super::super::constants::FIND_CLUSTERS_LIMIT;
use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

pub struct Find {
    data: web::Data<FindData>,
    logger: Logger,
    tracer: Arc<opentracingrust::Tracer>,
}

impl Find {
    pub fn new(interfaces: &mut Interfaces) -> Self {
        let data = FindData {
            store: interfaces.stores.primary.clone(),
        };
        Find {
            data: web::Data::new(data),
            logger: interfaces.logger.clone(),
            tracer: interfaces.tracing.tracer(),
        }
    }

    pub fn resource_default(&self) -> impl HttpServiceFactory {
        self.resource(web::resource("/find"))
    }

    pub fn resource_query(&self) -> impl HttpServiceFactory {
        self.resource(web::resource("/find/{query:.*?}"))
    }

    fn resource(&self, resource: Resource) -> impl HttpServiceFactory {
        let logger = self.logger.clone();
        let tracer = Arc::clone(&self.tracer);
        let tracer = TracingMiddleware::with_name(logger, tracer, "/clusters/find/{query}");
        resource
            .app_data(self.data.clone())
            .wrap(tracer)
            .route(web::get().to(responder))
    }
}

#[derive(Clone)]
struct FindData {
    store: Store,
}

async fn responder(data: web::Data<FindData>, request: HttpRequest) -> Result<impl Responder> {
    let path = request.match_info();
    let query = path.get("query").unwrap_or("").to_string();
    let mut request = request;
    let clusters = with_request_span(&mut request, |span| {
        let span = span.map(|span| span.context().clone());
        data.store
            .legacy()
            .find_clusters(query.to_string(), FIND_CLUSTERS_LIMIT, span)
            .with_context(|_| ErrorKind::PrimaryStoreQuery("find_clusters"))
    })?;
    let mut response = Vec::new();
    for cluster in clusters {
        let cluster = cluster.with_context(|_| ErrorKind::PrimaryStoreQuery("find_clusters"))?;
        response.push(cluster);
    }
    let response = HttpResponse::Ok().json(response);
    Ok(response)
}
