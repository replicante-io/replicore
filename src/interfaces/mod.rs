use slog::Logger;

use replicante_data_store::Store;

use super::Result;
use super::config::Config;


pub mod api;
pub mod metrics;
pub mod tracing;

use self::api::API;
use self::metrics::Metrics;
use self::tracing::Tracing;


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
    pub store: Store,
    pub tracing: Tracing,
}

impl Interfaces {
    /// Creates and configures interfaces.
    pub fn new(config: &Config, logger: Logger) -> Result<Interfaces> {
        let metrics = Metrics::new();
        let api = API::new(config.api.clone(), logger.clone(), &metrics);
        let store = Store::new(config.storage.clone(), logger.clone(), metrics.registry())?;
        let tracing = Tracing::new(config.tracing.clone(), logger.clone())?;
        Ok(Interfaces {
            api,
            metrics,
            store,
            tracing,
        })
    }

    /// Performs any final configuration and starts background threads.
    ///
    /// For example, the [`ApiInterface`] uses it to wrap the router into a server.
    pub fn run(&mut self) -> Result<()> {
        self.api.run()?;
        self.metrics.run()?;
        self.tracing.run()?;
        Ok(())
    }

    /// Waits for all interfaces to terminate.
    pub fn wait_all(&mut self) -> Result<()> {
        self.api.wait()?;
        self.metrics.wait()?;
        self.tracing.wait()?;
        Ok(())
    }
}


// *** Implement interfaces mocks for tests *** //
/// A container for mocks used by interfaces.
#[cfg(test)]
pub struct MockInterfaces {
}

#[cfg(test)]
impl Interfaces {
    /// Mock interfaces and wrap them in an `Interfaces` instance.
    ///
    /// This method will use a JSON logger to stdout.
    /// Use `Interfaces::mock_with_logger` to specify the logger.
    pub fn mock() -> (Interfaces, MockInterfaces) {
        let logger = super::logging::starter();
        Interfaces::mock_with_logger(logger)
    }

    /// Mock interfaces using the given logger and wrap them in an `Interfaces` instance.
    pub fn mock_with_logger(logger: Logger) -> (Interfaces, MockInterfaces) {
        let metrics = Metrics::mock();
        let api = API::mock(logger.clone(), &metrics);
        let tracing = Tracing::mock();

        let mock_store = ::replicante_data_store::mock::MockStore::new();
        let mock_store = ::std::sync::Arc::new(mock_store);
        let store = Store::mock(mock_store.clone());

        // Wrap things up.
        let mocks = MockInterfaces {};
        let interfaces = Interfaces {
            api,
            metrics,
            store,
            tracing,
        };
        (interfaces, mocks)
    }
}


#[cfg(test)]
mod tests {
    use super::Interfaces;

    #[test]
    fn instantiate_mocks() {
        let (interfaces, mocks) = Interfaces::mock();
        drop(interfaces);
        drop(mocks);
    }
}
