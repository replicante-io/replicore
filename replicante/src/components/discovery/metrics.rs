use prometheus::Counter;
use prometheus::Gauge;
use prometheus::Histogram;
use prometheus::HistogramOpts;
use prometheus::Opts;
use prometheus::Registry;

use slog::Logger;


lazy_static! {
    pub static ref DISCOVERY_COUNT: Counter = Counter::with_opts(
        Opts::new("replicore_discovery_loops", "Number of discovery runs started")
    ).expect("Failed to create DISCOVERY_COUNT");

    pub static ref DISCOVERY_DURATION: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "replicore_discovery_duration",
            "Duration (in seconds) of agent discovery runs"
        ).buckets(vec![0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 20.0, 40.0])
    ).expect("Failed to create DISCOVERY_DURATION");

    pub static ref DISCOVERY_ERRORS: Counter = Counter::with_opts(
        Opts::new("replicore_discovery_errors", "Number of errors during agent discovery")
    ).expect("Failed to create DISCOVERY_ERRORS");

    pub static ref DISCOVERY_SNAPSHOT_TRACKED_CLUSTERS: Gauge = Gauge::with_opts(
        Opts::new(
            "replicore_discovery_snapshot_tracked_clusters",
            "Number of clusters tracked by the discovery snapshots emission tracker"
        )
    ).expect("Failed to create DISCOVERY_SNAPSHOT_TRACKED_CLUSTERS");
}


/// Attemps to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    if let Err(error) = registry.register(Box::new(DISCOVERY_COUNT.clone())) {
        debug!(logger, "Failed to register DISCOVERY_COUNT"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(DISCOVERY_DURATION.clone())) {
        debug!(logger, "Failed to register DISCOVERY_DURATION"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(DISCOVERY_ERRORS.clone())) {
        debug!(logger, "Failed to register DISCOVERY_ERRORS"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(DISCOVERY_SNAPSHOT_TRACKED_CLUSTERS.clone())) {
        debug!(logger, "Failed to register DISCOVERY_SNAPSHOT_TRACKED_CLUSTERS"; "error" => ?error);
    }
}
