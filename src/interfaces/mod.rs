use slog::Logger;

use super::Result;
use super::config::Config;


pub mod api;
pub mod metrics;

use self::api::API;
use self::metrics::Metrics;


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
    pub api: API,
    pub metrics: Metrics,
}

impl Interfaces {
    /// Creates and configures interfaces.
    pub fn new(config: &Config, logger: Logger) -> Result<Interfaces> {
        let metrics = Metrics::new();
        let api = API::new(config.api.clone(), logger, &metrics);
        Ok(Interfaces {
            api,
            metrics,
        })
    }

    /// Performs any final configuration and starts background threads.
    ///
    /// For example, the [`ApiInterface`] uses it to wrap the router into a server.
    pub fn run(&mut self) -> Result<()> {
        self.api.run()?;
        self.metrics.run()?;
        Ok(())
    }

    /// Waits for all interfaces to terminate.
    pub fn wait_all(&mut self) -> Result<()> {
        self.api.wait()?;
        self.metrics.wait()?;
        Ok(())
    }
}
