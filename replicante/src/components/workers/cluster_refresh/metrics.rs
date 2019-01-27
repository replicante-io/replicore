use prometheus::CounterVec;
use prometheus::HistogramOpts;
use prometheus::HistogramVec;
use prometheus::Opts;
use prometheus::Registry;

use slog::Logger;


lazy_static! {
    pub static ref REFRESH_DURATION: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "replicore_cluster_refresh_duration",
            "Duration (in seconds) of a cluster state refresh task"
        ).buckets(vec![0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 20.0, 40.0]),
        &["cluster"]
    ).expect("Failed to create REFRESH_DURATION");

    pub static ref REFRESH_LOCKED: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_cluster_refresh_locked",
            "Number of times a cluster refresh task was abandoned because the cluster is locked"
        ),
        &["cluster"]
    ).expect("Failed to create REFRESH_LOCKED");
}


/// Attempts to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    if let Err(error) = registry.register(Box::new(REFRESH_DURATION.clone())) {
        debug!(logger, "Failed to register REFRESH_DURATION"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(REFRESH_LOCKED.clone())) {
        debug!(logger, "Failed to register REFRESH_LOCKED"; "error" => ?error);
    }
}
