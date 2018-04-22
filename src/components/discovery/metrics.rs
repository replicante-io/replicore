use prometheus::Counter;
use prometheus::Histogram;
use prometheus::HistogramOpts;
use prometheus::Opts;
use prometheus::Registry;

use slog::Logger;


lazy_static! {
    /// Counter for discovery cycles.
    pub static ref DISCOVERY_COUNT: Counter = Counter::with_opts(
        Opts::new("replicante_discovery_loops", "Number of discovery runs started")
    ).expect("Failed to create DISCOVERY_COUNT counter");

    /// Counter for discovery cycles that fail to fetch agents.
    pub static ref DISCOVERY_FETCH_ERRORS_COUNT: Counter = Counter::with_opts(
        Opts::new("replicante_discovery_fetch_errors", "Number of errors during agent discovery")
    ).expect("Failed to create DISCOVERY_FETCH_ERRORS_COUNT counter");

    /// Counter for discovery cycles that fail to process agents.
    pub static ref DISCOVERY_PROCESS_ERRORS_COUNT: Counter = Counter::with_opts(
        Opts::new(
            "replicante_discovery_process_errors",
            "Number of errors during processing of discovered agents"
        )
    ).expect("Failed to create DISCOVERY_PROCESS_ERRORS_COUNT counter");

    /// Observe duration of agent discovery.
    pub static ref DISCOVERY_DURATION: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "replicante_discovery_duration",
            "Duration (in seconds) of agent discovery runs"
        ).buckets(vec![0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 20.0, 40.0])
    ).expect("Failed to create DISCOVERY_DURATION histogram");
}


/// Attemps to register metrics with the Repositoy.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    if let Err(err) = registry.register(Box::new(DISCOVERY_COUNT.clone())) {
        let error = format!("{:?}", err);
        debug!(logger, "Failed to register DISCOVERY_COUNT"; "error" => error);
    }
    if let Err(err) = registry.register(Box::new(DISCOVERY_FETCH_ERRORS_COUNT.clone())) {
        let error = format!("{:?}", err);
        debug!(logger, "Failed to register DISCOVERY_FETCH_ERRORS_COUNT"; "error" => error);
    }
    if let Err(err) = registry.register(Box::new(DISCOVERY_PROCESS_ERRORS_COUNT.clone())) {
        let error = format!("{:?}", err);
        debug!(logger, "Failed to register DISCOVERY_PROCESS_ERRORS_COUNT"; "error" => error);
    }
    if let Err(err) = registry.register(Box::new(DISCOVERY_DURATION.clone())) {
        let error = format!("{:?}", err);
        debug!(logger, "Failed to register DISCOVERY_DURATION"; "error" => error);
    }
}
