use lazy_static::lazy_static;
use prometheus::Gauge;
use prometheus::GaugeVec;
use prometheus::Opts;
use prometheus::Registry;
use slog::debug;
use slog::Logger;

lazy_static! {
    pub static ref COMPONENTS_ENABLED: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_components_enabled",
            "Enabled status of components on this node (1 = enabled, 0 = disabled)"
        ),
        &["component", "type"]
    )
    .expect("Failed to create COMPONENTS_ENABLED gauge");
    pub static ref UPDATE_AVAILABLE: Gauge = Gauge::new(
        "replicore_updateable",
        "Set to 1 when an updateded version is available (checked at start only)",
    )
    .expect("Failed to create UPDATE_AVAILABLE gauge");
    pub static ref WORKERS_ENABLED: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_workers_enabled",
            "Enabled status of task workers on this node (1 = enabled, 0 = disabled)"
        ),
        &["worker"]
    )
    .expect("Failed to create WORKERS_ENABLED gauge");
}

/// Attemps to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    if let Err(error) = registry.register(Box::new(COMPONENTS_ENABLED.clone())) {
        debug!(logger, "Failed to register COMPONENTS_ENABLED"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(UPDATE_AVAILABLE.clone())) {
        debug!(logger, "Failed to register UPDATE_AVAILABLE"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(WORKERS_ENABLED.clone())) {
        debug!(logger, "Failed to register WORKERS_ENABLED"; "error" => ?error);
    }
}
