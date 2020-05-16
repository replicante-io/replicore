use prometheus::Registry;
use slog::Logger;

use replicante_util_actixweb::MetricsCollector;

lazy_static::lazy_static! {
    pub static ref REQUESTS: MetricsCollector = MetricsCollector::new("replicore");
}

/// Attemps to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    REQUESTS.register(logger, registry);
}
