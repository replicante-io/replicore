use slog::Logger;

use super::Config;
use super::Interfaces;
use super::Result;


/// A container for replicante components.
///
/// This container is useful to:
///
///   1. Have one argument passed arround for injection instead of many.
///   2. Store thread [`JoinHandle`]s to join on [`Drop`].
///
/// [`Drop`]: std/ops/trait.Drop.html
/// [`JoinHandle`]: std/thread/struct.JoinHandle.html
pub struct Components {
}

impl Components {
    /// Creates and configures components.
    pub fn new(_config: &Config, _logger: &Logger, _interfaces: &Interfaces) -> Result<Components> {
        Ok(Components {
        })
    }

    /// Performs any final configuration and starts background threads.
    pub fn run(&mut self) -> Result<()> {
        Ok(())
    }

    /// Waits for all interfaces to terminate.
    pub fn wait_all(&mut self) -> Result<()> {
        Ok(())
    }
}
