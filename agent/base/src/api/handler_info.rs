use iron::prelude::*;
use iron::Handler;
use iron::status;

use iron_json_response::JsonResponse;
use iron_json_response::JsonResponseMiddleware;

use super::super::AgentContainer;
use super::super::models::AgentVersion;
use super::super::models::DatastoreVersion;


/// Handler struct to implement the /api/v1/info endpoint.
pub struct InfoHandler {
    agent: AgentContainer,
    version: AgentVersion
}

impl InfoHandler {
    pub fn new(agent: AgentContainer, version: AgentVersion) -> Chain {
        let handler = InfoHandler { agent, version };
        let mut chain = Chain::new(handler);
        chain.link_after(JsonResponseMiddleware::new());
        chain
    }
}

impl Handler for InfoHandler {
    fn handle(&self, _: &mut Request) -> IronResult<Response> {
        let version = VersionInfo {
            datastore: self.agent.datastore_version(),
            version: self.version.clone()
        };
        let mut response = Response::new();
        response.set_mut(JsonResponse::json(version)).set_mut(status::Ok);
        Ok(response)
    }
}


/// Wrapps the agent and datastore versions for API response.
#[derive(Serialize)]
struct VersionInfo {
    datastore: DatastoreVersion,
    version: AgentVersion
}


#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use iron::Headers;
    use iron_test::request;
    use iron_test::response;

    use super::InfoHandler;
    use super::super::super::Agent;
    use super::super::super::models::AgentVersion;
    use super::super::super::models::DatastoreVersion;

    struct TestAgent {}
    impl Agent for TestAgent {
        fn datastore_version(&self) -> DatastoreVersion {
            DatastoreVersion::new("DB", "1.2.3")
        }
    }

    #[test]
    fn info_handler_returns_version() {
        let handler = InfoHandler::new(
            Arc::new(Box::new(TestAgent {})),
            AgentVersion::new("dcd", "1.2.3", "tainted")
        );
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
