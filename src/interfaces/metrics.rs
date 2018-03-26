use prometheus::Registry;

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
}
