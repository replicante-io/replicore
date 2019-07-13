use failure::ResultExt;
use iron::status;
use iron::Handler;
use iron::IronResult;
use iron::Request;
use iron::Response;
use iron::Set;
use iron_json_response::JsonResponse;
use router::Router;

use replicante_store_primary::store::Store as PrimaryStore;
use replicante_util_iron::request_span;

use crate::Error;
use crate::ErrorKind;

/// Cluster discovery (`/webui/cluster/:cluster/discovery`) handler.
pub struct Discovery {
    store: PrimaryStore,
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
        let span = request_span(req);
        let discovery = self
            .store
            .cluster(cluster.clone())
            .discovery(span.context().clone())
            .with_context(|_| ErrorKind::PrimaryStoreQuery("cluster.discovery"))
            .map_err(Error::from)?
            .ok_or_else(|| ErrorKind::ModelNotFound("ClusterDiscovery", cluster))
            .map_err(Error::from)?;
        let mut resp = Response::new();
        resp.set_mut(JsonResponse::json(discovery))
            .set_mut(status::Ok);
        Ok(resp)
    }
}

impl Discovery {
    pub fn new(store: PrimaryStore) -> Self {
        Discovery { store }
    }
}
