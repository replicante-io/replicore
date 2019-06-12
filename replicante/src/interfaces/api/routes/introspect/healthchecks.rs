use iron::status;
use iron::IronResult;
use iron::Request;
use iron::Response;
use iron::Set;
use iron_json_response::JsonResponse;
use serde_derive::Serialize;

use replicante_service_healthcheck::HealthResults;

use super::super::super::super::healthchecks::HealthResultsCache;

/// Report the result of the most reacent health checks.
pub struct Handler {
    cache: HealthResultsCache,
}

impl Handler {
    pub fn new(cache: HealthResultsCache) -> Handler {
        Handler { cache }
    }
}

impl ::iron::Handler for Handler {
    fn handle(&self, _: &mut Request) -> IronResult<Response> {
        let (instant, results) = self.cache.get();
        let age_secs = instant.elapsed().as_secs();
        let results = HealthInfo { age_secs, results };
        let mut resp = Response::new();
        resp.set_mut(JsonResponse::json(results))
            .set_mut(status::Ok);
        Ok(resp)
    }
}

#[derive(Serialize)]
struct HealthInfo {
    age_secs: u64,
    results: HealthResults,
}
