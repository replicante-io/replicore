use prometheus::Counter;
use prometheus::Histogram;
use prometheus::HistogramOpts;
use prometheus::Registry;
use slog::Logger;

lazy_static! {
    pub static ref AGGREGATE_DURATION: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "replicore_aggregate_duration",
            "Duration (in seconds) it takes to generate aggregated cluster data"
        )
        // Buckers: start = 1, next = prev * 10
        .buckets(vec![1.0, 10.0, 100.0, 1000.0, 10000.0, 100000.0]),
    )
    .expect("Failed to create replicore_fetcher_duration histogram");
    pub static ref AGGREGATE_ERRORS_COUNT: Counter = Counter::new(
        "replicore_fetcher_errors", "Number of fetchers errors"
    )
    .expect("Failed to create replicore_fetcher_errors counter");
}

pub fn register_metrics(logger: &Logger, registry: &Registry) {
    if let Err(error) = registry.register(Box::new(AGGREGATE_DURATION.clone())) {
        debug!(logger, "Failed to register AGGREGATE_DURATION"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(AGGREGATE_ERRORS_COUNT.clone())) {
        debug!(logger, "Failed to register AGGREGATE_ERRORS_COUNT"; "error" => ?error);
    }
}
