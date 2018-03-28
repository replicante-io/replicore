use prometheus::Registry;
use prometheus::process_collector::ProcessCollector;

use super::super::Result;


/// Interface for metrics collection.
///
/// This interface provides access to the [`Registry`] and to global metrics.
/// Other interfaces and components should register their metrics during initialisation.
pub struct Metrics {
    registry: Registry,
}

impl Metrics {
    /// Creates a new `Metrics` interface.
    pub fn new() -> Metrics {
        let registry = Registry::new();
        let process = ProcessCollector::for_self();
        registry.register(Box::new(process)).expect("Unable to register process metrics");
        Metrics { registry }
    }

    /// Access the metrics registery.
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// Noop method for standard interface.
    pub fn run(&self) -> Result<()> {
        Ok(())
    }

    /// Noop method for standard interface.
    pub fn wait(&self) -> Result<()> {
        Ok(())
    }

    /// Returns a `Metrics` instance usable as a mock.
    #[cfg(test)]
    pub fn mock() -> Metrics {
        Metrics::new()
    }
}
