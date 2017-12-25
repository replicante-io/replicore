use iron::prelude::*;
use iron::Handler;
use iron::status;

use iron_json_response::JsonResponse;
use iron_json_response::JsonResponseMiddleware;

use super::VersionInfo;


/// Handler struct to implement the /api/v1/info endpoint.
pub struct InfoHandler {
    version: VersionInfo
}

impl InfoHandler {
    pub fn new(version: VersionInfo) -> Chain {
        let handler = InfoHandler { version };
        let mut chain = Chain::new(handler);
        chain.link_after(JsonResponseMiddleware::new());
        chain
    }
}

impl Handler for InfoHandler {
    fn handle(&self, _: &mut Request) -> IronResult<Response> {
        let mut response = Response::new();
        response.set_mut(JsonResponse::json(&self.version)).set_mut(status::Ok);
        Ok(response)
    }
}


pub fn index(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "API endpoints mounted under /api/v1/")))
}

pub fn status(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "TODO")))
}


#[cfg(test)]
mod tests {

    use iron::Headers;
    use iron_test::request;
    use iron_test::response;

    use super::InfoHandler;
    use super::super::VersionInfo;

    #[test]
    fn index_points_to_api() {
        let response = request::get(
            "http://localhost:3000/",
            Headers::new(), &super::index
        ).unwrap();
        let result_body = response::extract_body_to_bytes(response);
        let result_body = String::from_utf8(result_body).unwrap();
        assert_eq!(result_body, "API endpoints mounted under /api/v1/");
    }

    #[test]
    fn info_handler_returns_version() {
        let handler = InfoHandler::new(VersionInfo::new(
            "DB", "1.2.3",
            "dcd", "1.2.3", "tainted"
        ));
        let response = request::get(
            "http://localhost:3000/api/v1/index",
            Headers::new(), &handler
        ).unwrap();
        let result_body = response::extract_body_to_bytes(response);
        let result_body = String::from_utf8(result_body).unwrap();
        let expected = r#"{"datastore":{"name":"DB","version":"1.2.3"},"version":{"checkout":"dcd","number":"1.2.3","taint":"tainted"}}"#;
        assert_eq!(result_body, expected);
    }
}
