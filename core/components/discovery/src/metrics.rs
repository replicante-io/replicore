use prometheus::Counter;
use prometheus::Histogram;
use prometheus::HistogramOpts;
use prometheus::Opts;
use prometheus::Registry;
use slog::debug;
use slog::Logger;

lazy_static::lazy_static! {
    pub static ref DISCOVERY_DURATION: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "replicore_discovery_duration",
            "Duration (in seconds) of pending discovery search and schedule cycles",
        )
        .buckets(vec![0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 20.0, 40.0])
    )
    .expect("Failed to create DISCOVERY_DURATION");
    pub static ref DISCOVERY_LOOP_COUNT: Counter = Counter::with_opts(Opts::new(
        "replicore_discovery_loops",
        "Number of pending discovery runs search and schedule cycles",
    ))
    .expect("Failed to create DISCOVERY_LOOP_COUNT");
    pub static ref DISCOVERY_LOOP_ERRORS: Counter = Counter::with_opts(Opts::new(
        "replicore_discovery_loop_errors",
        "Number of errors during pending discovery runs search and schedule cycles",
    ))
    .expect("Failed to create DISCOVERY_ERRORS");
    pub static ref DISCOVERY_SCHEDULE_COUNT: Counter = Counter::with_opts(Opts::new(
        "replicore_discovery_scheduled",
        "Number of pending discovery scheduled to run",
    ))
    .expect("Failed to create DISCOVERY_SCHEDULE_COUNT");
}

/// Attemps to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    if let Err(error) = registry.register(Box::new(DISCOVERY_DURATION.clone())) {
        debug!(logger, "Failed to register DISCOVERY_DURATION"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(DISCOVERY_LOOP_COUNT.clone())) {
        debug!(logger, "Failed to register DISCOVERY_LOOP_COUNT"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(DISCOVERY_LOOP_ERRORS.clone())) {
        debug!(logger, "Failed to register DISCOVERY_LOOP_ERRORS"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(DISCOVERY_SCHEDULE_COUNT.clone())) {
        debug!(logger, "Failed to register DISCOVERY_SCHEDULE_COUNT"; "error" => ?error);
    }
}
