use prometheus::CounterVec;
use prometheus::HistogramVec;
use prometheus::Registry;
use slog::Logger;

use replicante_util_iron::MetricsMiddleware;

lazy_static! {
    pub static ref MIDDLEWARE: (HistogramVec, CounterVec, CounterVec) =
        MetricsMiddleware::metrics("replicore");
}

/// Attemps to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    // Register the three middleware metrics.
    if let Err(error) = registry.register(Box::new(MIDDLEWARE.0.clone())) {
        debug!(logger, "Failed to register MIDDLEWARE.0"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(MIDDLEWARE.1.clone())) {
        debug!(logger, "Failed to register MIDDLEWARE.1"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(MIDDLEWARE.2.clone())) {
        debug!(logger, "Failed to register MIDDLEWARE.2"; "error" => ?error);
    }
}
