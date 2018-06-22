//! Module to define events related WebUI endpoints.
use iron::Handler;
use iron::IronResult;
use iron::Request;
use iron::Response;
use iron::Set;
use iron::status;
use iron_json_response::JsonResponse;

use replicante_data_store::Store;

use super::super::super::ResultExt;
use super::super::super::interfaces::Interfaces;


const FAIL_FETCH_EVENTS: &str = "Failed to fetch recent events";
const RECENT_EVENTS_LIMIT: u32 = 100;


/// Cluster discovery (`/webui/events`) handler.
pub struct Events {
    store: Store
}

impl Handler for Events {
    fn handle(&self, _req: &mut Request) -> IronResult<Response> {
        let events = self.store.recent_events(RECENT_EVENTS_LIMIT)
            .chain_err(|| FAIL_FETCH_EVENTS)?;
        let mut resp = Response::new();
        resp.set_mut(JsonResponse::json(events)).set_mut(status::Ok);
        Ok(resp)
    }
}

impl Events {
    pub fn attach(interfaces: &mut Interfaces) {
        let router = interfaces.api.router();
        let handler = Events { store: interfaces.store.clone() };
        router.get("/webui/events", handler, "webui_events");
    }
}
