use lazy_static::lazy_static;
use prometheus::Counter;
use prometheus::Histogram;
use prometheus::HistogramOpts;
use prometheus::Registry;
use slog::debug;
use slog::Logger;

lazy_static! {
    pub static ref ORCHESTRATE_DURATION: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "replicore_orchestrate_cluster_duration",
            "Duration (in seconds) of a cluster orchestration task"
        )
        .buckets(vec![0.5, 1.0, 2.5, 5.0, 7.0, 10.0, 20.0, 40.0]),
    )
    .expect("Failed to create ORCHESTRATE_DURATION");
    pub static ref ORCHESTRATE_LOCKED: Counter = Counter::new(
        "replicore_orchestrate_cluster_locked",
        "Number of times a cluster orchestration task was abandoned because the cluster is locked"
    )
    .expect("Failed to create ORCHESTRATE_LOCKED");
    pub static ref SETTINGS_DISABLED_COUNT: Counter = Counter::new(
        "replicore_orchestrate_cluster_disabled",
        "Number of times a cluster orchestration task was abandoned because the cluster is disabled"
    )
    .expect("Failed to create SETTINGS_DISABLED_COUNT");
}

/// Attempts to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    if let Err(error) = registry.register(Box::new(ORCHESTRATE_DURATION.clone())) {
        debug!(logger, "Failed to register ORCHESTRATE_DURATION"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(ORCHESTRATE_LOCKED.clone())) {
        debug!(logger, "Failed to register ORCHESTRATE_LOCKED"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(SETTINGS_DISABLED_COUNT.clone())) {
        debug!(logger, "Failed to register SETTINGS_DISABLED_COUNT"; "error" => ?error);
    }
}
