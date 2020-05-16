use actix_web::web;
use actix_web::Responder;
use prometheus::Registry;

use replicante_service_coordinator::Coordinator;
use replicante_util_actixweb::RootDescriptor;

use super::super::healthchecks::HealthResultsCache;
use super::APIRoot;
use crate::interfaces::api::AppConfigContext;

mod introspect;

/// Mount all API endpoints.
pub fn configure(
    cache: HealthResultsCache,
    coordinator: Coordinator,
    registry: Registry,
) -> impl Fn(&mut AppConfigContext) {
    let introspect = self::introspect::configure(cache, coordinator, registry);

    move |conf| {
        // Create the index root for each API root.
        let roots = [APIRoot::UnstableApi];
        for root in roots.iter() {
            root.and_then(&conf.context.flags, |root| {
                let resource = web::resource(root.prefix()).route(web::get().to(api_root));
                conf.app.service(resource);
            });
        }

        // Mount additional roots.
        introspect(conf);
    }
}

/// Render a simple message at the `/` of an API root (`/api/unstable`, `/api/v1`, ...).
async fn api_root() -> impl Responder {
    "Replicante API server".to_string()
}
