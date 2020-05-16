use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Resource;
use actix_web::Responder;
use serde_derive::Serialize;

use replicante_service_healthcheck::HealthResults;

use super::super::super::super::healthchecks::HealthResultsCache;

/// Report the result of the most reacent health checks.
pub struct HealthChecks {
    cache: HealthResultsCache,
}

impl HealthChecks {
    pub fn new(cache: HealthResultsCache) -> HealthChecks {
        HealthChecks { cache }
    }

    /// Return an `actix_web::Resource` to handle healthcheck requests.
    pub fn resource(&self) -> Resource {
        web::resource("/health")
            .data(self.cache.clone())
            .route(web::get().to(responder))
    }
}

#[derive(Serialize)]
struct HealthInfo {
    age_secs: u64,
    results: HealthResults,
}

async fn responder(cache: web::Data<HealthResultsCache>) -> impl Responder {
    let (instant, results) = cache.get();
    let age_secs = instant.elapsed().as_secs();
    let results = HealthInfo { age_secs, results };
    HttpResponse::Ok().json(results)
}
