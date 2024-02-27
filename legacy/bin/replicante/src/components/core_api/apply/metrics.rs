use lazy_static::lazy_static;
use prometheus::CounterVec;
use prometheus::HistogramOpts;
use prometheus::HistogramVec;
use prometheus::Opts;
use prometheus::Registry;
use slog::debug;
use slog::Logger;

lazy_static! {
    pub static ref APPLY_COUNT: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_apply_total",
            "Total number of objects applied through the API",
        ),
        &["apiVersion", "kind"],
    )
    .expect("Failed to create APPLY_COUNT");
    pub static ref APPLY_DURATION: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "replicore_apply_duration",
            "Duration (in seconds) of an apply operation"
        ),
        &["apiVersion", "kind"],
    )
    .expect("Failed to create APPLY_DURATION");
    pub static ref APPLY_ERROR: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_apply_error",
            "Number of attempts to apply objects that failed",
        ),
        &["apiVersion", "kind"],
    )
    .expect("Failed to create APPLY_ERROR");
    pub static ref APPLY_UNKNOWN: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_apply_unknown",
            "Number of attempts to apply unknown objects versions or kinds",
        ),
        &["apiVersion", "kind"],
    )
    .expect("Failed to create APPLY_UNKNOWN");
}

/// Attemps to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    if let Err(error) = registry.register(Box::new(APPLY_COUNT.clone())) {
        debug!(logger, "Failed to register APPLY_COUNT"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(APPLY_DURATION.clone())) {
        debug!(logger, "Failed to register APPLY_DURATION"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(APPLY_ERROR.clone())) {
        debug!(logger, "Failed to register APPLY_ERROR"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(APPLY_UNKNOWN.clone())) {
        debug!(logger, "Failed to register APPLY_UNKNOWN"; "error" => ?error);
    }
}
