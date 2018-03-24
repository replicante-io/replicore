use slog::Logger;

use super::Result;
use super::config::Config;


/// A container for replicante interfaces.
///
/// This container is useful to:
///
///   1. Have one argument passed arround for injection instead of many.
///   2. Store thread [`JoinHandle`]s to join on [`Drop`].
///
/// [`Drop`]: std/ops/trait.Drop.html
/// [`JoinHandle`]: std/thread/struct.JoinHandle.html
pub struct Interfaces {
    logger: Logger,
}

impl Interfaces {
    /// Creates and configures interfaces.
    pub fn new(config: &Config, logger: Logger) -> Result<Interfaces> {
        Ok(Interfaces { logger })
    }

    /// Performs any final configuration and starts background threads.
    ///
    /// For example, the [`ApiInterface`] uses it to wrap the router into a server.
    pub fn run(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Drop for Interfaces {
    fn drop(&mut self) {
        info!(self.logger, "Shutdown: cleaning up interfaces ...");
    }
}
