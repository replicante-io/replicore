use prometheus::Registry;

use replicante_coordinator::Coordinator;
use replicante_util_iron::MetricsHandler;
use replicante_util_iron::Router;

use super::super::super::healthchecks::HealthResultsCache;
use super::APIRoot;

mod healthchecks;
mod self_endpoint;
mod threads;
mod version;

pub fn mount(
    router: &mut Router,
    coordinator: Coordinator,
    registry: Registry,
    healthchecks: HealthResultsCache,
) {
    let healthchecks = healthchecks::Handler::new(healthchecks);
    let metrics = MetricsHandler::new(registry);
    let mut root = router.for_root(&APIRoot::UnstableIntrospect);
    root.get("/health", healthchecks, "/health");
    root.get("/metrics", metrics, "/metrics");
    root.get("/self", self_endpoint::Handler::new(coordinator), "/self");
    root.get("/threads", threads::handler, "/threads");
    root.get("/version", version::handler, "/version");
}
