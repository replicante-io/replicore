//! Module to define cluster related WebUI endpoints.
use iron::Handler;
use iron::IronResult;
use iron::Request;
use iron::Response;
use iron::Set;
use iron::status;
use iron_json_response::JsonResponse;
use router::Router;

use replicante_data_store::Store;

use super::super::super::Result;
use super::super::super::ResultExt;
use super::super::super::interfaces::Interfaces;


/// Cluster discovery (`/webui/cluster/:cluster/discovery`) handler.
pub struct Discovery {
    store: Store
}

impl Handler for Discovery {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let cluster: Result<String> = req.extensions.get::<Router>()
            .unwrap().find("cluster")
            .map(|cluster| String::from(cluster))
            .ok_or("Missing `cluster` parameter".into());
        let cluster = cluster?;
        let discovery = self.store.cluster_discovery(cluster)
            .chain_err(|| "Failed to fetch cluster discovery")?;
        let discovery: Result<_> = discovery.ok_or("Cluster not found".into());
        let discovery = discovery?;
        let mut resp = Response::new();
        resp.set_mut(JsonResponse::json(discovery)).set_mut(status::Ok);
        Ok(resp)
    }
}

impl Discovery {
    pub fn attach(interfaces: &mut Interfaces) {
        let router = interfaces.api.router();
        let handler = Discovery { store: interfaces.store.clone() };
        router.get("/webui/cluster/:cluster/discovery", handler, "webui_cluster_discovery");
    }
}


/// Cluster meta (`/webui/cluster/:cluster/meta`) handler.
pub struct Meta {
    store: Store
}

impl Handler for Meta {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let cluster: Result<String> = req.extensions.get::<Router>()
            .unwrap().find("cluster")
            .map(|cluster| String::from(cluster))
            .ok_or("Missing `cluster` parameter".into());
        let cluster = cluster?;
        let meta = self.store.cluster_meta(cluster)
            .chain_err(|| "Failed to fetch cluster metadata")?;
        let meta: Result<_> = meta.ok_or("Cluster not found".into());
        let meta = meta?;
        let mut resp = Response::new();
        resp.set_mut(JsonResponse::json(meta)).set_mut(status::Ok);
        Ok(resp)
    }
}

impl Meta {
    pub fn attach(interfaces: &mut Interfaces) {
        let router = interfaces.api.router();
        let handler = Meta { store: interfaces.store.clone() };
        router.get("/webui/cluster/:cluster/meta", handler, "webui_cluster_meta");
    }
}


//#[cfg(test)]
//mod tests {
//    mod top {
//        use std::sync::Arc;
//
//        use iron::Chain;
//        use iron::Headers;
//        use iron_json_response::JsonResponseMiddleware;
//        use iron_test::request;
//        use iron_test::response;
//
//        use replicante_data_models::webui::ClusterMeta;
//        use replicante_data_store::Store;
//        use replicante_data_store::mock::MockStore;
//
//        use super::super::Top;
//
//        fn mockstore() -> MockStore {
//            let mut mock_store = MockStore::new();
//            let c1 = ClusterMeta::new("c1", "mongo", 3);
//            let c2 = ClusterMeta::new("c2", "redis", 5);
//            mock_store.top_clusters = vec![c1, c2];
//            mock_store
//        }
//
//        fn handler(store: &Arc<MockStore>) -> Chain {
//            let store = Store::mock(Arc::clone(&store));
//            let handler = Top { store };
//            let mut handler = Chain::new(handler);
//            handler.link_after(JsonResponseMiddleware::new());
//            handler
//        }
//
//        #[test]
//        fn get_top_clusers() {
//            let mock_store = Arc::new(mockstore());
//            let handler = handler(&mock_store);
//            let response = request::get("http://host:16016/", Headers::new(), &handler).unwrap();
//            let result_body = response::extract_body_to_bytes(response);
//            let result_body = String::from_utf8(result_body).unwrap();
//            assert_eq!(result_body, r#"[{"name":"c1","kinds":["mongo"],"nodes":3},{"name":"c2","kinds":["redis"],"nodes":5}]"#);
//        }
//    }
//}
