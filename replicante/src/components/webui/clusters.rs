//! Module to define cluster related WebUI endpoints.
use failure::ResultExt;
use failure::err_msg;

use iron::Handler;
use iron::IronResult;
use iron::Request;
use iron::Response;
use iron::Set;
use iron::status;
use iron_json_response::JsonResponse;
use router::Router;

use replicante_data_store::Store;

use super::super::super::Error;
use super::super::super::ErrorKind;
use super::super::super::interfaces::Interfaces;


static FIND_CLUSTERS_LIMIT: u8 = 25;


/// Clusters find (`/webui/clusters/find`) handler.
pub struct Find {
    store: Store
}

impl Handler for Find {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let query = req.extensions.get::<Router>().unwrap().find("query").unwrap_or("");
        let clusters = self.store.find_clusters(query, FIND_CLUSTERS_LIMIT)
            .map_err(Error::from)
            .context(ErrorKind::Legacy(err_msg("failed to search clusters")))
            .map_err(Error::from)?;
        let mut resp = Response::new();
        resp.set_mut(JsonResponse::json(clusters)).set_mut(status::Ok);
        Ok(resp)
    }
}

impl Find {
    /// Attaches the handler for `/webui/clusters/top`.
    pub fn attach(interfaces: &mut Interfaces) {
        let router = interfaces.api.router();
        let handler_root = Find { store: interfaces.store.clone() };
        let handler_query = Find { store: interfaces.store.clone() };
        router.get("/webui/clusters/find", handler_root, "webui/clusters/find");
        router.get("/webui/clusters/find/:query", handler_query, "webui/clusters/find/query");
    }
}


/// Top clusters (`/webui/clusters/top`) handler.
pub struct Top {
    store: Store
}

impl Handler for Top {
    fn handle(&self, _: &mut Request) -> IronResult<Response> {
        let clusters = self.store.top_clusters().map_err(Error::from)
            .context(ErrorKind::Legacy(err_msg("could not fetch top clusters")))
            .map_err(Error::from)?;
        let mut resp = Response::new();
        resp.set_mut(JsonResponse::json(clusters)).set_mut(status::Ok);
        Ok(resp)
    }
}

impl Top {
    /// Attaches the handler for `/webui/clusters/top`.
    pub fn attach(interfaces: &mut Interfaces) {
        let router = interfaces.api.router();
        let handler = Top {
            store: interfaces.store.clone()
        };
        router.get("/webui/clusters/top", handler, "webui/clusters/top");
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

        use replicante_data_models::ClusterMeta;
        use replicante_data_store::Store;
        use replicante_data_store::mock::MockStore;

        use super::super::Top;

        fn mockstore() -> MockStore {
            let mock_store = MockStore::new();
            let c1 = ClusterMeta::new("c1", "mongo", 3);
            let c2 = ClusterMeta::new("c2", "redis", 5);
            mock_store.clusters_meta.lock().unwrap().insert("c1".into(), c1);
            mock_store.clusters_meta.lock().unwrap().insert("c2".into(), c2);
            mock_store
        }

        fn handler(store: &Arc<MockStore>) -> Chain {
            let store = Store::mock(Arc::clone(&store));
            let handler = Top { store };
            let mut handler = Chain::new(handler);
            handler.link_after(JsonResponseMiddleware::new());
            handler
        }

        #[test]
        fn get_top_clusers() {
            let mock_store = Arc::new(mockstore());
            let handler = handler(&mock_store);
            let response = request::get("http://host:16016/", Headers::new(), &handler).unwrap();
            let result_body = response::extract_body_to_bytes(response);
            let result_body = String::from_utf8(result_body).unwrap();
            assert_eq!(result_body, r#"[{"name":"c1","kinds":["mongo"],"nodes":3},{"name":"c2","kinds":["redis"],"nodes":5}]"#);
        }
    }
}
