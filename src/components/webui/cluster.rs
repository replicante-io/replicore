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
