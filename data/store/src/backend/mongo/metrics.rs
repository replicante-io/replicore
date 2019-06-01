use lazy_static::lazy_static;
use prometheus::CounterVec;
use prometheus::HistogramOpts;
use prometheus::HistogramVec;
use prometheus::Opts;
use prometheus::Registry;
use slog::debug;
use slog::Logger;

lazy_static! {
    pub static ref MONGODB_OP_ERRORS_COUNT: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_pristore_mongodb_operation_errors",
            "Number of MongoDB operations failed"
        ),
        &["operation"]
    )
    .expect("Failed to create replicante_mongodb_operation_errors counter");
    pub static ref MONGODB_OPS_COUNT: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_pristore_mongodb_operations",
            "Number of MongoDB operations issued"
        ),
        &["operation"]
    )
    .expect("Failed to create replicante_mongodb_operations counter");
    pub static ref MONGODB_OPS_DURATION: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "replicore_pristore_mongodb_operations_duration",
            "Duration (in seconds) of MongoDB operations"
        ),
        &["operation"]
    )
    .expect("Failed to create MONGODB_OPS_DURATION histogram");
}

pub fn register_metrics(logger: &Logger, registry: &Registry) {
    if let Err(error) = registry.register(Box::new(MONGODB_OPS_COUNT.clone())) {
        debug!(logger, "Failed to register MONGODB_OPS_COUNT"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(MONGODB_OP_ERRORS_COUNT.clone())) {
        debug!(logger, "Failed to register MONGODB_OP_ERRORS_COUNT"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(MONGODB_OPS_DURATION.clone())) {
        debug!(logger, "Failed to register MONGODB_OPS_DURATION"; "error" => ?error);
    }
}
