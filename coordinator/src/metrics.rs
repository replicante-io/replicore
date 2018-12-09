use prometheus::Registry;
use slog::Logger;


/// Attemps to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    super::backend::zookeeper::register_metrics(logger, registry);
}
