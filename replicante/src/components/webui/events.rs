//! Module to define events related WebUI endpoints.
use failure::ResultExt;
use failure::err_msg;

use iron::Handler;
use iron::IronResult;
use iron::Request;
use iron::Response;
use iron::Set;
use iron::status;
use iron_json_response::JsonResponse;

use replicante_data_store::EventsFilters;
use replicante_data_store::EventsOptions;
use replicante_data_store::Store;

use super::super::super::Error;
use super::super::super::ErrorKind;
use super::super::super::interfaces::Interfaces;


const FAIL_FETCH_EVENTS: &str = "Failed to fetch recent events";
const RECENT_EVENTS_LIMIT: i64 = 100;


/// Cluster discovery (`/webui/events`) handler.
pub struct Events {
    store: Store
}

impl Handler for Events {
    fn handle(&self, _req: &mut Request) -> IronResult<Response> {
        let mut options = EventsOptions::default();
        options.limit = Some(RECENT_EVENTS_LIMIT);
        options.reverse = true;
        let iter = self.store.events(EventsFilters::all(), options)
            .with_context(|_| ErrorKind::PrimaryStoreQuery("events"))
            .map_err(Error::from)?;
        let mut events = Vec::new();
        for event in iter {
            let event = event
                .context(ErrorKind::Legacy(err_msg(FAIL_FETCH_EVENTS)))
                .map_err(Error::from)?;
            events.push(event);
        }

        let mut resp = Response::new();
        resp.set_mut(JsonResponse::json(events)).set_mut(status::Ok);
        Ok(resp)
    }
}

impl Events {
    pub fn attach(interfaces: &mut Interfaces) {
        let router = interfaces.api.router();
        let handler = Events { store: interfaces.store.clone() };
        router.get("/webui/events", handler, "webui/events");
    }
}
