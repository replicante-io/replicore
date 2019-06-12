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

use replicante_data_store::store::Store;

use super::super::super::interfaces::api::APIRoot;
use super::super::super::interfaces::Interfaces;
use super::super::super::Error;
use super::super::super::ErrorKind;

/// Cluster discovery (`/webui/cluster/:cluster/discovery`) handler.
pub struct Discovery {
    store: Store,
}

impl Handler for Discovery {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let cluster = req
            .extensions
            .get::<Router>()
            .expect("Iron Router extension not found")
            .find("cluster")
            .map(String::from)
            .ok_or_else(|| ErrorKind::APIRequestParameterNotFound("cluster"))
            .map_err(Error::from)?;
        let discovery = self
            .store
            .cluster(cluster.clone())
            .discovery(None)
            .with_context(|_| ErrorKind::PrimaryStoreQuery("cluster_discovery"))
            .map_err(Error::from)?
            .ok_or_else(|| ErrorKind::ModelNotFound("cluster_discovery", cluster))
            .map_err(Error::from)?;
        let mut resp = Response::new();
        resp.set_mut(JsonResponse::json(discovery))
            .set_mut(status::Ok);
        Ok(resp)
    }
}

impl Discovery {
    pub fn attach(interfaces: &mut Interfaces) {
        let mut router = interfaces.api.router_for(&APIRoot::UnstableWebUI);
        let handler = Discovery {
            store: interfaces.store.clone(),
        };
        router.get(
            "/cluster/:cluster/discovery",
            handler,
            "/cluster/:cluster/discovery",
        );
    }
}

/// Cluster meta (`/webui/cluster/:cluster/meta`) handler.
pub struct Meta {
    store: Store,
}

impl Handler for Meta {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let cluster = req
            .extensions
            .get::<Router>()
            .expect("Iron Router extension not found")
            .find("cluster")
            .map(String::from)
            .ok_or_else(|| ErrorKind::APIRequestParameterNotFound("cluster"))
            .map_err(Error::from)?;
        let meta = self
            .store
            .legacy()
            .cluster_meta(cluster.clone(), None)
            .with_context(|_| ErrorKind::PrimaryStoreQuery("cluster_meta"))
            .map_err(Error::from)?
            .ok_or_else(|| ErrorKind::ModelNotFound("cluster_meta", cluster))
            .map_err(Error::from)?;
        let mut resp = Response::new();
        resp.set_mut(JsonResponse::json(meta)).set_mut(status::Ok);
        Ok(resp)
    }
}

impl Meta {
    pub fn attach(interfaces: &mut Interfaces) {
        let mut router = interfaces.api.router_for(&APIRoot::UnstableWebUI);
        let handler = Meta {
            store: interfaces.store.clone(),
        };
        router.get("/cluster/:cluster/meta", handler, "/cluster/:cluster/meta");
    }
}
