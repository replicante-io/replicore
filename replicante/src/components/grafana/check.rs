//! Module to define cluster related WebUI endpoints.
use iron::Handler;
use iron::IronResult;
use iron::Request;
use iron::Response;
use iron::status;

use super::Interfaces;

/// Grafana check endpoint (`/api/v1/grafana`) handler.
pub struct Check {}

impl Handler for Check {
    fn handle(&self, _req: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "Grafana SimpleJson Annotations API endpoints")))
    }
}

impl Check {
    pub fn attach(interfaces: &mut Interfaces) {
        let router = interfaces.api.router();
        let handler = Check {};
        router.get("/api/v1/grafana", handler, "api/v1/grafana");
    }
}
