use actix_web::web;
use actix_web::Resource;
use prometheus::Registry;

use replicante_service_coordinator::Coordinator;
use replicante_util_actixweb::MetricsExporter;
use replicante_util_actixweb::RootDescriptor;

mod healthchecks;
mod my_self;
mod threads;
mod version;

use super::APIRoot;
use super::HealthResultsCache;
use crate::interfaces::api::AppConfigContext;

use self::healthchecks::HealthChecks;
use self::my_self::MySelf;

pub fn configure(
    cache: HealthResultsCache,
    coordinator: Coordinator,
    registry: Registry,
) -> impl Fn(&mut AppConfigContext) {
    let health = HealthChecks::new(cache);
    let my_self = MySelf::new(coordinator);

    move |conf| {
        APIRoot::UnstableIntrospect.and_then(&conf.context.flags, |root| {
            let prefix = root.prefix();
            conf.scoped_service(prefix, health.resource());
            conf.scoped_service(prefix, metrics(registry.clone()));
            conf.scoped_service(prefix, my_self.resource());
            conf.scoped_service(prefix, self::threads::threads);
            conf.scoped_service(prefix, self::version::version);
        });
    }
}

fn metrics(registry: Registry) -> Resource {
    let metrics = MetricsExporter::factory(registry);
    web::resource("/metrics").route(web::get().to(metrics))
}
