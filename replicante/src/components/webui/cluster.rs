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


/// Cluster discovery (`/webui/cluster/:cluster/discovery`) handler.
pub struct Discovery {
    store: Store
}

impl Handler for Discovery {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let cluster = req.extensions.get::<Router>()
            .unwrap().find("cluster")
            .map(String::from)
            .ok_or_else(|| ErrorKind::Legacy(err_msg("missing `cluster` parameter")))
            .map_err(Error::from)?;
        let discovery = self.store.cluster_discovery(cluster.clone())
            .with_context(|_| ErrorKind::PrimaryStoreQuery("cluster_discovery"))
            .map_err(Error::from)?
            .ok_or_else(|| ErrorKind::ModelNotFound("cluster_discovery", cluster))
            .map_err(Error::from)?;
        let mut resp = Response::new();
        resp.set_mut(JsonResponse::json(discovery)).set_mut(status::Ok);
        Ok(resp)
    }
}

impl Discovery {
    pub fn attach(interfaces: &mut Interfaces) {
        let router = interfaces.api.router();
        let handler = Discovery { store: interfaces.store.clone() };
        router.get("/webui/cluster/:cluster/discovery", handler, "webui/cluster/discovery");
    }
}


/// Cluster meta (`/webui/cluster/:cluster/meta`) handler.
pub struct Meta {
    store: Store
}

impl Handler for Meta {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let cluster = req.extensions.get::<Router>()
            .unwrap().find("cluster")
            .map(String::from)
            .ok_or_else(|| ErrorKind::Legacy(err_msg("missing `cluster` parameter")))
            .map_err(Error::from)?;
        let meta = self.store.cluster_meta(cluster.clone())
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
        let router = interfaces.api.router();
        let handler = Meta { store: interfaces.store.clone() };
        router.get("/webui/cluster/:cluster/meta", handler, "webui/cluster/meta");
    }
}
