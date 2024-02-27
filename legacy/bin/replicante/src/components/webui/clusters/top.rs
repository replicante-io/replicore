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

pub struct Top {
    data: web::Data<TopData>,
    logger: Logger,
    tracer: Arc<opentracingrust::Tracer>,
}

impl Top {
    pub fn new(interfaces: &mut Interfaces) -> Self {
        let data = TopData {
            store: interfaces.stores.primary.clone(),
        };
        Top {
            data: web::Data::new(data),
            logger: interfaces.logger.clone(),
            tracer: interfaces.tracing.tracer(),
        }
    }

    pub fn resource(&self) -> impl HttpServiceFactory {
        let logger = self.logger.clone();
        let tracer = Arc::clone(&self.tracer);
        let tracer = TracingMiddleware::with_name(logger, tracer, "/clusters/top");
        web::resource("/top")
            .app_data(self.data.clone())
            .wrap(tracer)
            .route(web::get().to(responder))
    }
}

#[derive(Clone)]
struct TopData {
    store: Store,
}

async fn responder(data: web::Data<TopData>, request: HttpRequest) -> Result<impl Responder> {
    let mut request = request;
    let clusters = with_request_span(&mut request, |span| {
        let span = span.map(|span| span.context().clone());
        data.store
            .legacy()
            .top_clusters(span)
            .with_context(|_| ErrorKind::PrimaryStoreQuery("top_clusters"))
    })?;
    let mut response = Vec::new();
    for cluster in clusters {
        let cluster = cluster.with_context(|_| ErrorKind::PrimaryStoreQuery("top_clusters"))?;
        response.push(cluster);
    }
    let response = HttpResponse::Ok().json(response);
    Ok(response)
}

#[cfg(test)]
mod tests {
    use actix_web::test::call_and_read_body_json;
    use actix_web::test::init_service;
    use actix_web::test::TestRequest;
    use actix_web::App;

    use replicante_models_core::cluster::ClusterMeta;

    use crate::interfaces::test_support::MockInterfaces;

    fn large_cluster() -> ClusterMeta {
        let mut cluster = ClusterMeta::new("c2", "redis");
        cluster.kinds = vec!["redis".into()];
        cluster.nodes = 5;
        cluster
    }

    fn small_cluster() -> ClusterMeta {
        let mut cluster = ClusterMeta::new("c1", "mongo");
        cluster.kinds = vec!["mongo".into()];
        cluster.nodes = 3;
        cluster
    }

    fn mock_cluster_meta(mocks: &MockInterfaces, cluster: ClusterMeta) {
        let name = cluster.cluster_id.clone();
        mocks
            .stores
            .primary
            .state
            .lock()
            .unwrap()
            .clusters_meta
            .insert(name, cluster);
    }

    #[actix_rt::test]
    async fn get_top_clusers() {
        let mocks = MockInterfaces::mock_quietly();
        mock_cluster_meta(&mocks, small_cluster());
        mock_cluster_meta(&mocks, large_cluster());

        let mut interfaces = mocks.interfaces();
        let top = super::Top::new(&mut interfaces);
        let app = App::new().service(top.resource());
        let mut app = init_service(app).await;

        let request = TestRequest::get().uri("/top").to_request();
        let response: Vec<ClusterMeta> = call_and_read_body_json(&mut app, request).await;

        let cluster_1 = small_cluster();
        let cluster_2 = large_cluster();
        assert_eq!(response, vec![cluster_2, cluster_1]);
    }
}
