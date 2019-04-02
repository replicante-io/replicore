//! Module that defines a set of core routes for the API interface.
use prometheus::Registry;

use replicante_util_iron::MetricsHandler;

use super::APIVersion;
use super::RouterBuilder;

mod index;
mod version;

/// Mount core API route handlers.
pub fn mount(router: &mut RouterBuilder, registry: Registry) {
    let metrics = MetricsHandler::new(registry);
    let mut unstable = router.for_version(APIVersion::Unstable);
    unstable.get("/", index::handler, "index");
    unstable.get("/metrics", metrics, "/metrics");
    unstable.get("/version", version::handler, "/version");
}
