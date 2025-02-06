//! API service to inspect and manage a lease.
use actix_web::web::Data;
use actix_web::web::Query;
use actix_web::HttpResponse;

use replicore_coordinator::LeaseHandle;

use crate::api::Error;

/// Query parameters for the lease step-down endpoint.
#[derive(Debug, serde::Deserialize)]
struct StepDownQuery {
    /// Delay, in seconds, before the lease can attempt re-acquiring.
    delay: u64,
}

/// API service to inspect and manage a lease.
pub fn service(path: &str, lease: LeaseHandle) -> actix_web::Scope {
    actix_web::web::scope(path)
        .app_data(Data::new(lease))
        .service(status)
        .service(step_down)
}

/// Inspect the current state of the lease.
#[actix_web::get("")]
async fn status(lease: Data<LeaseHandle>) -> Result<HttpResponse, Error> {
    let info = serde_json::json!({
        "lease_id": lease.id(),
        "state": lease.state(),
    });
    Ok(HttpResponse::Ok().json(info))
}

/// Perform a step-down operation for the lease.
#[actix_web::post("/step-down")]
async fn step_down(
    args: Query<StepDownQuery>,
    lease: Data<LeaseHandle>,
) -> Result<HttpResponse, Error> {
    let delay = std::time::Duration::from_secs(args.delay);
    lease.step_down(delay).await?;
    Ok(crate::api::done())
}
