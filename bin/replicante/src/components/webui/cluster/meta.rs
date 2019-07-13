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

/// Cluster meta (`/webui/cluster/:cluster/meta`) handler.
pub struct Meta {
    store: PrimaryStore,
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
        let span = request_span(req);
        let meta = self
            .store
            .legacy()
            .cluster_meta(cluster.clone(), span.context().clone())
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
    pub fn new(store: PrimaryStore) -> Self {
        Meta { store }
    }
}
