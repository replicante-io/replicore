use prometheus::Registry;
use prometheus::Opts;
use prometheus::GaugeVec;
use slog::Logger;


lazy_static! {
    pub static ref COMPONENTS_ENABLED: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_components_enabled",
            "Enabled status of components on this node (1 = enabled, 0 = disabled)"
        ),
        &["component", "type"]
    ).expect("Failed to create COMPONENTS_ENABLED gauge");

    pub static ref WORKERS_ENABLED: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_workers_enabled",
            "Enabled status of task workers on this node (1 = enabled, 0 = disabled)"
        ),
        &["worker"]
    ).expect("Failed to create WORKERS_ENABLED gauge");
}


/// Attemps to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    if let Err(error) = registry.register(Box::new(COMPONENTS_ENABLED.clone())) {
        debug!(logger, "Failed to register COMPONENTS_ENABLED"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(WORKERS_ENABLED.clone())) {
        debug!(logger, "Failed to register WORKERS_ENABLED"; "error" => ?error);
    }
}
