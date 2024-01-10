use prometheus::Counter;
use prometheus::Opts;
use prometheus::Registry;
use slog::debug;
use slog::Logger;

lazy_static::lazy_static! {
    pub static ref DISCOVER_CLUSTER_SETTINGS_COUNT: Counter = Counter::with_opts(Opts::new(
        "replicore_discover_cluster_settings",
        "Number of ClusterSettings record created while discovering clusters",
    ))
    .expect("Failed to create DISCOVER_CLUSTER_SETTINGS_COUNT");
    pub static ref DISCOVER_DISABLED_COUNT: Counter = Counter::with_opts(Opts::new(
        "replicore_discover_disabled",
        "Number of discovery tasks skipped because the discovery was disabled",
    ))
    .expect("Failed to create DISCOVER_DISABLED_COUNT");
}

/// Attemps to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    if let Err(error) = registry.register(Box::new(DISCOVER_CLUSTER_SETTINGS_COUNT.clone())) {
        debug!(logger, "Failed to register DISCOVER_CLUSTER_SETTINGS_COUNT"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(DISCOVER_DISABLED_COUNT.clone())) {
        debug!(logger, "Failed to register DISCOVER_DISABLED_COUNT"; "error" => ?error);
    }
}
