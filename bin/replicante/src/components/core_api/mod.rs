use slog::Logger;

use replicante_util_upkeep::Upkeep;

use super::Component;
use crate::interfaces::Interfaces;
use crate::Result;

mod apply;
mod cluster;

pub use apply::register_metrics;

/// Component to mount replicante core API endpoints.
pub struct CoreAPI {}

impl CoreAPI {
    pub fn new(logger: Logger, interfaces: &mut Interfaces) -> CoreAPI {
        let apply = self::apply::configure(&logger, interfaces);
        let cluster = self::cluster::configure(&logger, interfaces);
        interfaces.api.configure(apply);
        interfaces.api.configure(cluster);
        CoreAPI {}
    }
}

impl Component for CoreAPI {
    fn run(&mut self, _: &mut Upkeep) -> Result<()> {
        Ok(())
    }
}
