use iron::status;
use iron::IronResult;
use iron::Request;
use iron::Response;
use iron::Set;
use iron_json_response::JsonResponse;

use replicante_coordinator::Coordinator;

/// Report information about the node itself.
pub struct Handler {
    coordinator: Coordinator,
}

impl Handler {
    pub fn new(coordinator: Coordinator) -> Handler {
        Handler { coordinator }
    }
}

impl ::iron::Handler for Handler {
    fn handle(&self, _: &mut Request) -> IronResult<Response> {
        let info = self.coordinator.node_id();
        let mut resp = Response::new();
        resp.set_mut(JsonResponse::json(info))
            .set_mut(status::Ok);
        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
//    use iron::Chain;
//    use iron::Headers;
//    use iron_test::request;
//    use iron_test::response;
//    use iron_json_response::JsonResponseMiddleware;
//
//    use super::handler;
//
//    #[test]
//    fn get_index() {
//        let mut chain = Chain::new(&handler);
//        chain.link_after(JsonResponseMiddleware::new());
//        let response = request::get("http://host:16016/", Headers::new(), &chain).unwrap();
//        let result_body = response::extract_body_to_bytes(response);
//        let result_body = String::from_utf8(result_body).unwrap();
//        assert_eq!(result_body, format!(
//            r#"{{"commit":"{}","taint":"{}","version":"{}"}}"#,
//            env!("GIT_BUILD_HASH"), env!("GIT_BUILD_TAINT"), env!("CARGO_PKG_VERSION")
//        ));
//    }
}
