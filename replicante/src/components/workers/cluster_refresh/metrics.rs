//use prometheus::Counter;
//use prometheus::Opts;
use prometheus::Registry;

use slog::Logger;


lazy_static! {
    //pub static ref DISCOVERY_PROCESS_ERRORS_COUNT: Counter = Counter::with_opts(
    //    Opts::new(
    //        "replicante_discovery_process_errors",
    //        "Number of errors during processing of discovered agents"
    //    )
    //).expect("Failed to create DISCOVERY_PROCESS_ERRORS_COUNT counter");
}


/// Attemps to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(_logger: &Logger, _registry: &Registry) {
    //if let Err(error) = registry.register(Box::new(DISCOVERY_PROCESS_ERRORS_COUNT.clone())) {
    //    debug!(logger, "Failed to register DISCOVERY_PROCESS_ERRORS_COUNT"; "error" => ?error);
    //}
}
