use prometheus::CounterVec;
use prometheus::Opts;
use prometheus::Registry;
use slog::Logger;


lazy_static! {
    /// Counter for fetcher errors by cluster.
    pub static ref FETCHER_ERRORS_COUNT: CounterVec = CounterVec::new(
        Opts::new("replicante_fetchers_errors", "Number of fetchers errors"),
        &["cluster"]
    ).expect("Failed to create replicante_fetchers_errors counter");
}


/// Attemps to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    if let Err(err) = registry.register(Box::new(FETCHER_ERRORS_COUNT.clone())) {
        let error = format!("{:?}", err);
        debug!(logger, "Failed to register FETCHER_ERRORS_COUNT"; "error" => error);
    }
}
