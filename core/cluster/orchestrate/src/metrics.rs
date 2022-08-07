use lazy_static::lazy_static;
use prometheus::Counter;
use prometheus::Histogram;
use prometheus::HistogramOpts;
use prometheus::Registry;
use slog::debug;
use slog::Logger;

lazy_static! {
    pub static ref ACTIONS_SYNCED: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "replicore_cluster_orchestrate_actions_synced",
            "Distribution of how many actions are synced to a node during orchestration",
        )
        .buckets(vec![1.0, 5.0, 15.0, 30.0, 60.0, 90.0, 120.0]),
    )
    .expect("Failed to create ACTIONS_SYNCED histogram");
    pub static ref ACTION_SCHEDULE_DUPLICATE: Counter = Counter::new(
        "replicore_cluster_orchestrate_actions_schedule_duplicate",
        "Number of duplicate action scheduling attempts",
    )
    .expect("Failed to create ACTION_SCHEDULE_DUPLICATE counter");
    pub static ref ACTION_SCHEDULE_ERROR: Counter = Counter::new(
        "replicore_cluster_orchestrate_actions_schedule_error",
        "Number of errors scheduling actions",
    )
    .expect("Failed to create ACTION_SCHEDULE_ERROR counter");
    pub static ref ACTION_SCHEDULE_TOTAL: Counter = Counter::new(
        "replicore_cluster_orchestrate_actions_schedule_total",
        "Total number of action scheduling attempts",
    )
    .expect("Failed to create ACTION_SCHEDULE_TOTAL counter");
    pub static ref AGGREGATE_ERRORS_COUNT: Counter = Counter::new(
        "replicore_cluster_orchestrate_sync_errors",
        "Number of errors during cluster orchestration",
    )
    .expect("Failed to create AGGREGATE_ERRORS_COUNT counter");
    pub static ref AGGREGATE_DURATION: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "replicore_cluster_orchestrate_aggregation_duration",
            "Duration (in seconds) of a cluster orchestrate aggregation cycle",
        )
        // Buckers: start = 1, next = prev + (idx) * 0.5
        .buckets(vec![1.0, 1.5, 2.5, 4.0, 6.0, 8.5, 11.5, 15.0]),
    )
    .expect("Failed to create AGGREGATE_DURATION histogram");
    pub static ref SYNC_DURATION: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "replicore_cluster_orchestrate_sync_duration",
            "Duration (in seconds) of a cluster orchestrate sync cycle",
        )
        // Buckers: start = 1, next = prev + (idx) * 0.5
        .buckets(vec![1.0, 1.5, 2.5, 4.0, 6.0, 8.5, 11.5, 15.0]),
    )
    .expect("Failed to create SYNC_DURATION histogram");
    pub static ref SYNC_ERRORS_COUNT: Counter = Counter::new(
        "replicore_cluster_orchestrate_sync_errors",
        "Number of errors during cluster orchestration",
    )
    .expect("Failed to create SYNC_ERRORS_COUNT counter");
}

/// Attempts to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    if let Err(error) = registry.register(Box::new(ACTIONS_SYNCED.clone())) {
        debug!(logger, "Failed to register ACTIONS_SYNCED"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(ACTION_SCHEDULE_DUPLICATE.clone())) {
        debug!(logger, "Failed to register ACTION_SCHEDULE_DUPLICATE"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(ACTION_SCHEDULE_ERROR.clone())) {
        debug!(logger, "Failed to register ACTION_SCHEDULE_ERROR"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(ACTION_SCHEDULE_TOTAL.clone())) {
        debug!(logger, "Failed to register ACTION_SCHEDULE_TOTAL"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(AGGREGATE_ERRORS_COUNT.clone())) {
        debug!(logger, "Failed to register AGGREGATE_ERRORS_COUNT"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(AGGREGATE_DURATION.clone())) {
        debug!(logger, "Failed to register AGGREGATE_DURATION"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(SYNC_DURATION.clone())) {
        debug!(logger, "Failed to register SYNC_DURATION"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(SYNC_ERRORS_COUNT.clone())) {
        debug!(logger, "Failed to register SYNC_ERRORS_COUNT"; "error" => ?error);
    }
}
