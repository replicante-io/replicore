use lazy_static::lazy_static;
use prometheus::Counter;
use prometheus::Histogram;
use prometheus::HistogramOpts;
use prometheus::Registry;
use slog::debug;
use slog::Logger;

lazy_static! {
    pub static ref FETCHER_ACTIONS_SYNCED: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "replicore_fetcher_actions_synced",
            "Distribution of how many actions are fetched to sync a node",
        )
        .buckets(vec![1.0, 5.0, 15.0, 30.0, 60.0, 90.0, 120.0]),
    )
    .expect("Failed to create FETCHER_ACTIONS_SYNCED histogram");
    pub static ref FETCHER_ACTION_SCHEDULE_DUPLICATE: Counter = Counter::new(
        "replicore_fetcher_action_schedule_duplicate",
        "Number of duplicate action scheduling attempts",
    )
    .expect("Failed to create FETCHER_ACTION_SCHEDULE_DUPLICATE counter");
    pub static ref FETCHER_ACTION_SCHEDULE_ERROR: Counter = Counter::new(
        "replicore_fetcher_action_schedule_error",
        "Number of errors scheduling actions",
    )
    .expect("Failed to create FETCHER_ACTION_SCHEDULE_ERROR counter");
    pub static ref FETCHER_ACTION_SCHEDULE_TOTAL: Counter = Counter::new(
        "replicore_fetcher_action_schedule_total",
        "Total number of action scheduling attempts",
    )
    .expect("Failed to create FETCHER_ACTION_SCHEDULE_TOTAL counter");
    pub static ref FETCHER_DURATION: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "replicore_fetcher_duration",
            "Duration (in seconds) of a cluster refresh"
        )
        // Buckers: start = 1, next = prev + (idx) * 0.5
        .buckets(vec![1.0, 1.5, 2.5, 4.0, 6.0, 8.5, 11.5, 15.0]),
    )
    .expect("Failed to create FETCHER_DURATION histogram");
    pub static ref FETCHER_ERRORS_COUNT: Counter = Counter::new(
        "replicore_fetcher_errors", "Number of fetchers errors"
    )
    .expect("Failed to create FETCHER_ERRORS_COUNT counter");
}

/// Attemps to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    if let Err(error) = registry.register(Box::new(FETCHER_ACTIONS_SYNCED.clone())) {
        debug!(logger, "Failed to register FETCHER_ACTIONS_SYNCED"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(FETCHER_ACTION_SCHEDULE_DUPLICATE.clone())) {
        debug!(logger, "Failed to register FETCHER_ACTION_SCHEDULE_DUPLICATE"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(FETCHER_ACTION_SCHEDULE_ERROR.clone())) {
        debug!(logger, "Failed to register FETCHER_ACTION_SCHEDULE_ERROR"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(FETCHER_ACTION_SCHEDULE_TOTAL.clone())) {
        debug!(logger, "Failed to register FETCHER_ACTION_SCHEDULE_TOTAL"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(FETCHER_DURATION.clone())) {
        debug!(logger, "Failed to register FETCHER_DURATION"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(FETCHER_ERRORS_COUNT.clone())) {
        debug!(logger, "Failed to register FETCHER_ERRORS_COUNT"; "error" => ?error);
    }
}
