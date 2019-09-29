use lazy_static::lazy_static;
use prometheus::Counter;
use prometheus::Histogram;
use prometheus::HistogramOpts;
use prometheus::Registry;
use slog::debug;
use slog::Logger;

lazy_static! {
    pub static ref FETCHER_ACTIONS_CHUNKS: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "replicore_fetcher_actions_chunks",
            "Distribution of how many action ID chunks are needed to sync a node",
        )
        .buckets(vec![1.0, 2.0, 4.0, 8.0, 10.0]),
    )
    .expect("Failed to create FETCHER_ACTIONS_CHUNKS histogram");
    pub static ref FETCHER_ACTIONS_SYNCED: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "replicore_fetcher_actions_synced",
            "Distribution of how many actions are fetched to sync a node",
        )
        .buckets(vec![1.0, 5.0, 15.0, 30.0, 60.0, 90.0, 120.0]),
    )
    .expect("Failed to create FETCHER_ACTIONS_SYNCED histogram");
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
    if let Err(error) = registry.register(Box::new(FETCHER_ACTIONS_CHUNKS.clone())) {
        debug!(logger, "Failed to register FETCHER_ACTIONS_CHUNKS"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(FETCHER_ACTIONS_SYNCED.clone())) {
        debug!(logger, "Failed to register FETCHER_ACTIONS_SYNCED"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(FETCHER_DURATION.clone())) {
        debug!(logger, "Failed to register FETCHER_DURATION"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(FETCHER_ERRORS_COUNT.clone())) {
        debug!(logger, "Failed to register FETCHER_ERRORS_COUNT"; "error" => ?error);
    }
}
