//! Module that defines a set of core routes for the API interface.
use prometheus::Registry;

use replicante_util_iron::MetricsHandler;

use super::RouterBuilder;


mod index;
mod version;


/// Mount core API route handlers.
pub fn mount(router: &mut RouterBuilder, registry: Registry) {
    let metrics = MetricsHandler::new(registry);
    router.get("/", index::handler, "index");
    router.get("/api/v1/metrics", metrics, "v1/metrics");
    router.get("/api/v1/version", version::handler, "v1/version");
}
