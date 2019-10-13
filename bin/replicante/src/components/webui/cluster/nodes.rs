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

/// Cluster discovery (`/webui/cluster/:cluster/nodes`) handler.
pub struct Nodes {
    store: PrimaryStore,
}

impl Handler for Nodes {
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

        let mut nodes = Vec::new();
        let iter = self
            .store
            .nodes(cluster)
            .iter(span.context().clone())
            .with_context(|_| ErrorKind::PrimaryStoreQuery("nodes.iter"))
            .map_err(Error::from)?;
        for node in iter {
            let node = node
                .with_context(|_| ErrorKind::Deserialize("node record", "Node"))
                .map_err(Error::from)?;
            nodes.push(node);
        }

        let mut resp = Response::new();
        resp.set_mut(JsonResponse::json(nodes)).set_mut(status::Ok);
        Ok(resp)
    }
}

impl Nodes {
    pub fn new(store: PrimaryStore) -> Self {
        Nodes { store }
    }
}
