use prometheus::Counter;
use prometheus::Histogram;
use prometheus::HistogramOpts;
use prometheus::Opts;
use prometheus::Registry;
use slog::debug;
use slog::Logger;

lazy_static::lazy_static! {
    pub static ref DURATION: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "replicore_discovery_scheduler_duration",
            "Duration (in seconds) of pending discovery search and schedule cycles",
        )
        .buckets(vec![0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 20.0, 40.0])
    )
    .expect("Failed to create DURATION");
    pub static ref LOOP_COUNT: Counter = Counter::with_opts(Opts::new(
        "replicore_discovery_scheduler_loops",
        "Number of pending discovery runs search and schedule cycles",
    ))
    .expect("Failed to create LOOP_COUNT");
    pub static ref LOOP_ERRORS: Counter = Counter::with_opts(Opts::new(
        "replicore_discovery_scheduler_loop_errors",
        "Number of errors during pending discovery runs search and schedule cycles",
    ))
    .expect("Failed to create ERRORS");
    pub static ref SCHEDULE_COUNT: Counter = Counter::with_opts(Opts::new(
        "replicore_discovery_scheduler_scheduled",
        "Number of pending discovery scheduled to run",
    ))
    .expect("Failed to create SCHEDULE_COUNT");
}

/// Attemps to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    if let Err(error) = registry.register(Box::new(DURATION.clone())) {
        debug!(logger, "Failed to register DURATION"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(LOOP_COUNT.clone())) {
        debug!(logger, "Failed to register LOOP_COUNT"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(LOOP_ERRORS.clone())) {
        debug!(logger, "Failed to register LOOP_ERRORS"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(SCHEDULE_COUNT.clone())) {
        debug!(logger, "Failed to register SCHEDULE_COUNT"; "error" => ?error);
    }
}
