//! Module to define cluster related WebUI endpoints.
use iron::status;
use iron::Handler;
use iron::IronResult;
use iron::Request;
use iron::Response;

use super::super::super::interfaces::api::APIRoot;
use super::Interfaces;

/// Grafana check endpoint (`/grafana`) handler.
pub struct Check {}

impl Handler for Check {
    fn handle(&self, _req: &mut Request) -> IronResult<Response> {
        Ok(Response::with((
            status::Ok,
            "Grafana SimpleJson Annotations API endpoints",
        )))
    }
}

impl Check {
    pub fn attach(interfaces: &mut Interfaces) {
        let mut router = interfaces.api.router_for(&APIRoot::UnstableAPI);
        let handler = Check {};
        router.get("/grafana", handler, "/grafana");
    }
}
