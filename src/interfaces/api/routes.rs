//! Module that defines a set of core handlers for the API interface.
use iron::IronResult;
use iron::Request;
use iron::Response;
use iron::status;


/// Root index (`/`) handler.
pub fn root_index(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "Replicante API server")))
}
