use lazy_static::lazy_static;
use prometheus::CounterVec;
use prometheus::Opts;
use prometheus::Registry;
use slog::debug;
use slog::Logger;

lazy_static! {
    pub static ref DISCOVERY_ERRORS: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_discovery_errors",
            "Number of cluster discovery request errors"
        ),
        &["backend"]
    )
    .expect("Failed to create DISCOVERY_ERRORS counter");
    pub static ref DISCOVERY_TOTAL: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_discovery_total",
            "Number of cluster discovery requests"
        ),
        &["backend"]
    )
    .expect("Failed to create DISCOVERY_TOTAL counter");
}

/// Attemps to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
///
/// **This method should be called before performing any discovery**.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    if let Err(error) = registry.register(Box::new(DISCOVERY_ERRORS.clone())) {
        debug!(logger, "Failed to register DISCOVERY_ERRORS"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(DISCOVERY_TOTAL.clone())) {
        debug!(logger, "Failed to register DISCOVERY_TOTAL"; "error" => ?error);
    }
}
