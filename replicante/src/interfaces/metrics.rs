use prometheus::Registry;
use prometheus::process_collector::ProcessCollector;
use slog::Logger;

use super::super::Result;


/// Attemps to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    // Register default process metircs.
    let process = ProcessCollector::for_self();
    if let Err(error) = registry.register(Box::new(process)) {
        debug!(logger, "Failed to register PROCESS metrics"; "error" => ?error);
    }
}


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

    /// Returns a `Metrics` instance usable as a mock.
    #[cfg(test)]
    pub fn mock() -> Metrics {
        Metrics::new()
    }
}
