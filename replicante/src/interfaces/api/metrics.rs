use prometheus::Registry;
use prometheus::HistogramVec;
use prometheus::CounterVec;
use slog::Logger;

use replicante_util_iron::MetricsMiddleware;


lazy_static! {
    pub static ref MIDDLEWARE: (HistogramVec, CounterVec, CounterVec) = {
        MetricsMiddleware::metrics("replicante")
    };
}


/// Attemps to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    // Register the three middleware metrics.
    if let Err(err) = registry.register(Box::new(MIDDLEWARE.0.clone())) {
        let error = format!("{:?}", err);
        debug!(logger, "Failed to register MIDDLEWARE.0"; "error" => error);
    }
    if let Err(err) = registry.register(Box::new(MIDDLEWARE.1.clone())) {
        let error = format!("{:?}", err);
        debug!(logger, "Failed to register MIDDLEWARE.1"; "error" => error);
    }
    if let Err(err) = registry.register(Box::new(MIDDLEWARE.2.clone())) {
        let error = format!("{:?}", err);
        debug!(logger, "Failed to register MIDDLEWARE.2"; "error" => error);
    }
}
