use prometheus::Counter;
use prometheus::Registry;
use slog::Logger;


lazy_static! {
    pub static ref NB_LOCK_ACQUIRE_FAIL: Counter = Counter::new(
        "replicore_coordinator_nb_lock_acquire_fail",
        "Number of non-blocking lock acquire operations that failed"
    ).expect("Failed to create NB_LOCK_ACQUIRE_FAIL counter");

    pub static ref NB_LOCK_ACQUIRE_TOTAL: Counter = Counter::new(
        "replicore_coordinator_nb_lock_acquire_total",
        "Total number of non-blocking lock acquire operations"
    ).expect("Failed to create NB_LOCK_ACQUIRE_TOTAL counter");

    pub static ref NB_LOCK_DROP_FAIL: Counter = Counter::new(
        "replicore_coordinator_nb_lock_drop_fail",
        "Number of non-blocking lock release-on-drop operations that failed"
    ).expect("Failed to create NB_LOCK_DROP_FAIL counter");

    pub static ref NB_LOCK_DROP_TOTAL: Counter = Counter::new(
        "replicore_coordinator_nb_lock_drop_total",
        "Total number of non-blocking lock release-on-drop operations"
    ).expect("Failed to create NB_LOCK_DROP_TOTAL counter");

    pub static ref NB_LOCK_LOST: Counter = Counter::new(
        "replicore_coordinator_nb_lock_lost",
        "Number of non-blocking locks lost (as reported by the backend)"
    ).expect("Failed to create NB_LOCK_LOST counter");

    pub static ref NB_LOCK_RELEASE_FAIL: Counter = Counter::new(
        "replicore_coordinator_nb_lock_release_fail",
        "Number of non-blocking lock release operations that failed"
    ).expect("Failed to create NB_LOCK_RELEASE_FAIL counter");

    pub static ref NB_LOCK_RELEASE_TOTAL: Counter = Counter::new(
        "replicore_coordinator_nb_lock_release_total",
        "Total number of non-blocking lock release operations"
    ).expect("Failed to create NB_LOCK_RELEASE_TOTAL counter");
}


/// Attemps to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    if let Err(err) = registry.register(Box::new(NB_LOCK_ACQUIRE_FAIL.clone())) {
        debug!(logger, "Failed to register NB_LOCK_ACQUIRE_FAIL"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(NB_LOCK_ACQUIRE_TOTAL.clone())) {
        debug!(logger, "Failed to register NB_LOCK_ACQUIRE_TOTAL"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(NB_LOCK_DROP_FAIL.clone())) {
        debug!(logger, "Failed to register NB_LOCK_DROP_FAIL"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(NB_LOCK_DROP_TOTAL.clone())) {
        debug!(logger, "Failed to register NB_LOCK_DROP_TOTAL"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(NB_LOCK_LOST.clone())) {
        debug!(logger, "Failed to register NB_LOCK_LOST"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(NB_LOCK_RELEASE_FAIL.clone())) {
        debug!(logger, "Failed to register NB_LOCK_RELEASE_FAIL"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(NB_LOCK_RELEASE_TOTAL.clone())) {
        debug!(logger, "Failed to register NB_LOCK_RELEASE_TOTAL"; "error" => ?err);
    }
    super::backend::zookeeper::register_metrics(logger, registry);
}
