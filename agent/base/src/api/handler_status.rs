use iron::prelude::*;
use iron::Handler;
use iron::status;

use iron_json_response::JsonResponse;
use iron_json_response::JsonResponseMiddleware;

use super::super::AgentContainer;
use super::super::models::Shard;


/// Handler implementing the /api/v1/status endpoint.
pub struct StatusHandler {
    agent: AgentContainer
}

impl StatusHandler {
    pub fn new(agent: AgentContainer) -> Chain {
        let handler = StatusHandler { agent };
        let mut chain = Chain::new(handler);
        chain.link_after(JsonResponseMiddleware::new());
        chain
    }
}

impl Handler for StatusHandler {
    fn handle(&self, _: &mut Request) -> IronResult<Response> {
        let shards = StatusRespone {
            shards: self.agent.shards()?
        };
        let mut response = Response::new();
        response.set_mut(JsonResponse::json(&shards)).set_mut(status::Ok);
        Ok(response)
    }
}


/// Wrapps the shards info for API response.
#[derive(Serialize)]
struct StatusRespone {
    shards: Vec<Shard>
}


#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use iron::Headers;
    use iron::IronError;
    use iron_test::request;
    use iron_test::response;

    use super::StatusHandler;
    use super::super::super::Agent;
    use super::super::super::AgentError;
    use super::super::super::AgentResult;

    use super::super::super::models::DatastoreVersion;
    use super::super::super::models::Shard;
    use super::super::super::models::ShardRole;

    struct TestAgent {}

    impl Agent for TestAgent {
        fn datastore_version(&self) -> AgentResult<DatastoreVersion> {
            Err(AgentError::GenericError(String::from("Not Needed")))
        }

        fn shards(&self) -> AgentResult<Vec<Shard>> {
            Ok(vec![
               Shard::new("test-shard", ShardRole::Primary, 1, 2)
            ])
        }
    }

    fn request_get(agent: Box<Agent + Send + Sync>) -> Result<String, IronError> {
        let handler = StatusHandler::new(Arc::new(agent));
        request::get(
            "http://localhost:3000/api/v1/status",
            Headers::new(), &handler
        )
        .map(|response| {
            let body = response::extract_body_to_bytes(response);
            String::from_utf8(body).unwrap()
        })
    }

    #[test]
    fn status_retruns_shards() {
        let result = request_get(Box::new(TestAgent {})).unwrap();
        assert_eq!(result, r#"{"shards":[{"id":"test-shard","lag":1,"last_op":2,"role":"Primary"}]}"#);
    }
}
