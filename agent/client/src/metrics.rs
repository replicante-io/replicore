use prometheus::CounterVec;
use prometheus::HistogramOpts;
use prometheus::HistogramVec;
use prometheus::Opts;
use prometheus::Registry;

use slog::Logger;

lazy_static! {
    pub static ref CLIENT_HTTP_STATUS: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_agentclient_http_status",
            "Number of HTTP status for an endpoint"
        ),
        &["endpoint", "status"]
    )
    .expect("Failed to create CLIENT_HTTP_STATUS counter");
    pub static ref CLIENT_OP_ERRORS_COUNT: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_agentclient_operation_errors",
            "Number of agent operations failed"
        ),
        &["endpoint"]
    )
    .expect("Failed to create CLIENT_OP_ERRORS_COUNT counter");
    pub static ref CLIENT_OPS_COUNT: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_agentclient_operations",
            "Number of agent operations issued"
        ),
        &["endpoint"]
    )
    .expect("Failed to create CLIENT_OPS_COUNT counter");
    pub static ref CLIENT_OPS_DURATION: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "replicore_agentclient_operations_duration",
            "Duration (in seconds) of agent operations"
        ),
        &["endpoint"]
    )
    .expect("Failed to create CLIENT_OPS_DURATION histogram");
    pub static ref CLIENT_TIMEOUT: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_agentclient_timeout",
            "Number of agent operations that timed out"
        ),
        &["endpoint"]
    )
    .expect("Failed to create CLIENT_TIMEOUT counter");
}

/// Attemps to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
///
/// **This method should be called before using any client**.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    if let Err(error) = registry.register(Box::new(CLIENT_HTTP_STATUS.clone())) {
        debug!(logger, "Failed to register CLIENT_HTTP_STATUS"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(CLIENT_OP_ERRORS_COUNT.clone())) {
        debug!(logger, "Failed to register CLIENT_OP_ERRORS_COUNT"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(CLIENT_OPS_COUNT.clone())) {
        debug!(logger, "Failed to register CLIENT_OPS_COUNT"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(CLIENT_OPS_DURATION.clone())) {
        debug!(logger, "Failed to register CLIENT_OPS_DURATION"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(CLIENT_TIMEOUT.clone())) {
        debug!(logger, "Failed to register CLIENT_TIMEOUT"; "error" => ?error);
    }
}
