use prometheus::Counter;
use prometheus::CounterVec;
use prometheus::HistogramOpts;
use prometheus::HistogramVec;
use prometheus::Opts;
use prometheus::Registry;
use slog::Logger;

lazy_static! {
    pub static ref ZOO_CLEANUP_COUNT: Counter = Counter::new(
        "replicore_coordinator_zookeeper_cleanup",
        "Number of znodes deleted by the background cleaner"
    )
    .expect("Failed to create ZOO_CLEANUP_COUNT counter");
    pub static ref ZOO_CONNECTION_COUNT: Counter = Counter::new(
        "replicore_coordinator_zookeeper_connect",
        "Number of connections to the zookeeper ensemble since the process started"
    )
    .expect("Failed to create ZOO_CONNECTION_COUNT counter");
    pub static ref ZOO_NB_LOCK_DELETED: Counter = Counter::new(
        "replicore_coordinator_zookeeper_nb_lock_deleted",
        "Number of non-blocking locks lost because their znode was deleted"
    )
    .expect("Failed to create ZOO_NB_LOCK_DELETED counter");
    pub static ref ZOO_NB_LOCK_LOST: Counter = Counter::new(
        "replicore_coordinator_zookeeper_nb_lock_lost",
        "Number of non-blocking locks lost because the owner session expired"
    )
    .expect("Failed to create ZOO_NB_LOCK_LOST counter");
    pub static ref ZOO_OP_DURATION: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "replicore_coordinator_zookeeper_op_duration",
            "Duration (in seconds) of Zookeeper operations"
        ),
        &["operation"]
    )
    .expect("Failed to create ZOO_OP_DURATION histogram");
    pub static ref ZOO_OP_ERRORS_COUNT: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_coordinator_zookeeper_op_errors",
            "Number of Zookeeper operations that failed"
        ),
        &["operation"]
    )
    .expect("Failed to create ZOO_OP_ERRORS_COUNT counter");
    pub static ref ZOO_TIMEOUTS_COUNT: Counter = Counter::new(
        "replicore_coordinator_zookeeper_timeouts",
        "Number of operations that failed due to timeouts"
    )
    .expect("Failed to create ZOO_TIMEOUTS_COUNT counter");
}

/// Attemps to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    if let Err(err) = registry.register(Box::new(ZOO_CLEANUP_COUNT.clone())) {
        debug!(logger, "Failed to register ZOO_CLEANUP_COUNT"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(ZOO_CONNECTION_COUNT.clone())) {
        debug!(logger, "Failed to register ZOO_CONNECTION_COUNT"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(ZOO_NB_LOCK_DELETED.clone())) {
        debug!(logger, "Failed to register ZOO_NB_LOCK_DELETED"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(ZOO_NB_LOCK_LOST.clone())) {
        debug!(logger, "Failed to register ZOO_NB_LOCK_LOST"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(ZOO_OP_DURATION.clone())) {
        debug!(logger, "Failed to register ZOO_OP_DURATION"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(ZOO_OP_ERRORS_COUNT.clone())) {
        debug!(logger, "Failed to register ZOO_OP_ERRORS_COUNT"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(ZOO_TIMEOUTS_COUNT.clone())) {
        debug!(logger, "Failed to register ZOO_TIMEOUTS_COUNT"; "error" => ?err);
    }
}
