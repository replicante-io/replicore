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

use replicante_store_primary::store::Store as PrimaryStore;
use replicante_store_view::store::events::EventsFilters;
use replicante_store_view::store::events::EventsOptions;
use replicante_store_view::store::Store as ViewStore;
use replicante_util_iron::request_span;

use super::constants::RECENT_EVENTS_LIMIT;
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
    pub fn new(store: PrimaryStore) -> Self {
        Discovery { store }
    }
}

/// Cluster events (`/webui/cluster/:cluster/events`) handler.
pub struct Events {
    store: ViewStore,
}

impl Handler for Events {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let cluster = req
            .extensions
            .get::<Router>()
            .expect("Iron Router extension not found")
            .find("cluster")
            .map(String::from)
            .ok_or_else(|| ErrorKind::APIRequestParameterNotFound("cluster"))
            .map_err(Error::from)?;

        let mut filters = EventsFilters::all();
        filters.cluster_id = Some(cluster);
        let mut options = EventsOptions::default();
        options.limit = Some(RECENT_EVENTS_LIMIT);
        options.reverse = true;

        let span = request_span(req);
        let iter = self
            .store
            .events()
            .range(filters, options, span.context().clone())
            .with_context(|_| ErrorKind::PrimaryStoreQuery("events"))
            .map_err(Error::from)?;
        let mut events = Vec::new();
        for event in iter {
            let event = event
                .with_context(|_| ErrorKind::Deserialize("event record", "Event"))
                .map_err(Error::from)?;
            events.push(event);
        }

        let mut resp = Response::new();
        resp.set_mut(JsonResponse::json(events)).set_mut(status::Ok);
        Ok(resp)
    }
}

impl Events {
    pub fn new(store: ViewStore) -> Self {
        Events { store }
    }
}

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
