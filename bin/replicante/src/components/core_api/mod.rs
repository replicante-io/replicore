use slog::Logger;

use replicante_util_upkeep::Upkeep;

use super::Component;
use crate::interfaces::Interfaces;
use crate::Result;

mod apply;
mod catalogue;
mod cluster;
mod discovery_settings;
mod namespace;
mod platform;

pub use apply::register_metrics;

/// Component to mount replicante core API endpoints.
pub struct CoreAPI {}

impl CoreAPI {
    pub fn new(logger: Logger, interfaces: &mut Interfaces) -> CoreAPI {
        let apply = self::apply::configure(&logger, interfaces);
        let catalogue = self::catalogue::configure(&logger, interfaces);
        let cluster = self::cluster::configure(&logger, interfaces);
        let discovery_settings = self::discovery_settings::configure(&logger, interfaces);
        let namespace = self::namespace::configure(&logger, interfaces);
        let platform = self::platform::configure(&logger, interfaces);
        interfaces.api.configure(apply);
        interfaces.api.configure(catalogue);
        interfaces.api.configure(cluster);
        interfaces.api.configure(discovery_settings);
        interfaces.api.configure(namespace);
        interfaces.api.configure(platform);
        CoreAPI {}
    }
}

impl Component for CoreAPI {
    fn run(&mut self, _: &mut Upkeep) -> Result<()> {
        Ok(())
    }
}
