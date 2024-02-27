use replicante_util_actixweb::RootDescriptor;
use replicante_util_upkeep::Upkeep;

use super::Component;
use crate::interfaces::api::APIRoot;
use crate::interfaces::Interfaces;
use crate::Result;

mod annotations;
mod check;

/// Component to mount grafana endpoints.
///
/// These endpoints are designed to provide an Annotations backend for the Grafana
/// [SimpleJson](https://grafana.com/plugins/grafana-simple-json-datasource) plugin.
pub struct Grafana {}

impl Grafana {
    pub fn new(interfaces: &mut Interfaces) -> Grafana {
        let annotations = self::annotations::Annotations::new(interfaces);
        interfaces.api.configure(move |conf| {
            APIRoot::UnstableApi.and_then(&conf.context.flags, |root| {
                let scope = actix_web::web::scope("/grafana")
                    .service(self::check::check)
                    .service(annotations.resource());
                conf.scoped_service(root.prefix(), scope);
            });
        });
        Grafana {}
    }
}

impl Component for Grafana {
    /// Noop method for standard interface.
    fn run(&mut self, _: &mut Upkeep) -> Result<()> {
        Ok(())
    }
}
