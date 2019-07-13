use failure::ResultExt;
use iron::status;
use iron::Handler;
use iron::IronResult;
use iron::Request;
use iron::Response;
use iron::Set;
use iron_json_response::JsonResponse;
use router::Router;

use replicante_store_view::store::events::EventsFilters;
use replicante_store_view::store::events::EventsOptions;
use replicante_store_view::store::Store as ViewStore;
use replicante_util_iron::request_span;

use super::super::constants::RECENT_EVENTS_LIMIT;
use crate::Error;
use crate::ErrorKind;

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
            .with_context(|_| ErrorKind::PrimaryStoreQuery("events.range"))
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
