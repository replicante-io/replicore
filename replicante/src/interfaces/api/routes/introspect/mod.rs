use prometheus::Registry;

use replicante_util_iron::MetricsHandler;
use replicante_util_iron::Router;

use super::APIRoot;

mod threads;
mod version;

pub fn mount(router: &mut Router, registry: Registry) {
    let metrics = MetricsHandler::new(registry);
    let mut root = router.for_root(APIRoot::UnstableIntrospect);
    root.get("/metrics", metrics, "/metrics");
    root.get("/threads", threads::handler, "/threads");
    root.get("/version", version::handler, "/version");
}
