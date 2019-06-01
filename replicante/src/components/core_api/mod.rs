use slog::Logger;

use replicante_util_upkeep::Upkeep;

use super::super::interfaces::Interfaces;
use super::super::Result;

mod cluster;

/// Component to mount replicante core API endpoints.
pub struct CoreAPI {}

impl CoreAPI {
    pub fn new(logger: Logger, interfaces: &mut Interfaces) -> CoreAPI {
        self::cluster::attach(logger, interfaces);
        CoreAPI {}
    }

    /// Noop method for standard interface.
    pub fn run(&self, _: &mut Upkeep) -> Result<()> {
        Ok(())
    }
}
