//! Module to define cluster related WebUI endpoints.
use failure::ResultExt;

use iron::status;
use iron::Handler;
use iron::IronResult;
use iron::Request;
use iron::Response;
use iron::Set;
use iron_json_response::JsonResponse;
use router::Router;

use replicante_store_primary::store::Store;
use replicante_util_iron::request_span;

use super::constants::FIND_CLUSTERS_LIMIT;
use crate::Error;
use crate::ErrorKind;

/// Clusters find (`/webui/clusters/find`) handler.
pub struct Find {
    store: Store,
}

impl Handler for Find {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let span_context = request_span(req).context().clone();
        let query = req
            .extensions
            .get::<Router>()
            .expect("Iron Router extension not found")
            .find("query")
            .unwrap_or("");
        let clusters = self
            .store
            .legacy()
            .find_clusters(query.to_string(), FIND_CLUSTERS_LIMIT, span_context)
            .with_context(|_| ErrorKind::PrimaryStoreQuery("find_clusters"))
            .map_err(Error::from)?;
        let mut response = Vec::new();
        for cluster in clusters {
            let cluster = cluster
                .with_context(|_| ErrorKind::PrimaryStoreQuery("find_clusters"))
                .map_err(Error::from)?;
            response.push(cluster);
        }
        let mut resp = Response::new();
        resp.set_mut(JsonResponse::json(response))
            .set_mut(status::Ok);
        Ok(resp)
    }
}

impl Find {
    pub fn new(store: Store) -> Self {
        Find { store }
    }
}

/// Top clusters (`/webui/clusters/top`) handler.
pub struct Top {
    store: Store,
}

impl Handler for Top {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let span = request_span(req);
        let clusters = self
            .store
            .legacy()
            .top_clusters(span.context().clone())
            .with_context(|_| ErrorKind::PrimaryStoreQuery("top_clusters"))
            .map_err(Error::from)?;
        let mut response = Vec::new();
        for cluster in clusters {
            let cluster = cluster
                .with_context(|_| ErrorKind::PrimaryStoreQuery("top_clusters"))
                .map_err(Error::from)?;
            response.push(cluster);
        }
        let mut resp = Response::new();
        resp.set_mut(JsonResponse::json(response))
            .set_mut(status::Ok);
        Ok(resp)
    }
}

impl Top {
    pub fn new(store: Store) -> Self {
        Top { store }
    }
}

#[cfg(test)]
mod tests {
    mod top {
        use std::sync::Arc;

        use iron::Chain;
        use iron::Headers;
        use iron_json_response::JsonResponseMiddleware;
        use iron_test::request;
        use iron_test::response;
        use opentracingrust::tracers::NoopTracer;
        use opentracingrust::Tracer;

        use replicante_models_core::ClusterMeta;
        use replicante_store_primary::mock::Mock as MockStore;
        use replicante_util_iron::mock_request_span;

        use super::super::Top;

        fn mockstore() -> MockStore {
            let mock_store = MockStore::default();
            let mut c1 = ClusterMeta::new("c1", "mongo");
            c1.kinds = vec!["mongo".into()];
            c1.nodes = 3;
            let mut c2 = ClusterMeta::new("c2", "redis");
            c2.kinds = vec!["redis".into()];
            c2.nodes = 5;
            mock_store
                .state
                .lock()
                .unwrap()
                .clusters_meta
                .insert("c1".into(), c1);
            mock_store
                .state
                .lock()
                .unwrap()
                .clusters_meta
                .insert("c2".into(), c2);
            mock_store
        }

        fn handler(mock: &MockStore, tracer: Arc<Tracer>) -> Chain {
            let store = mock.store();
            let handler = Top { store };
            let mut handler = Chain::new(mock_request_span(tracer, handler));
            handler.link_after(JsonResponseMiddleware::new());
            handler
        }

        #[test]
        fn get_top_clusers() {
            let mock_store = mockstore();
            let (tracer, _) = NoopTracer::new();
            let tracer = Arc::new(tracer);
            let handler = handler(&mock_store, tracer);
            let response = request::get("http://host:16016/", Headers::new(), &handler).unwrap();
            let result_body = response::extract_body_to_bytes(response);
            let result: Vec<ClusterMeta> = serde_json::from_slice(&result_body).unwrap();
            let mut c1 = ClusterMeta::new("c1", "mongo");
            c1.kinds = vec!["mongo".into()];
            c1.nodes = 3;
            let mut c2 = ClusterMeta::new("c2", "redis");
            c2.kinds = vec!["redis".into()];
            c2.nodes = 5;
            assert_eq!(result, vec![c2, c1]);
        }
    }
}
