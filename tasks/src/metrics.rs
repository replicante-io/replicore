use lazy_static::lazy_static;
use prometheus::Counter;
use prometheus::CounterVec;
use prometheus::Opts;
use prometheus::Registry;
use slog::debug;
use slog::Logger;

lazy_static! {
    pub static ref TASK_ACK_ERRORS: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_tasks_ack_errors",
            "Number of task ack operations that failed"
        ),
        &["queue", "op"]
    )
    .expect("Failed to create TASK_ACK_ERRORS counter");
    pub static ref TASK_ACK_TOTAL: CounterVec = CounterVec::new(
        Opts::new("replicore_tasks_ack_total", "Number of tasks acked"),
        &["queue", "op"]
    )
    .expect("Failed to create TASK_ACK_TOTAL counter");
    pub static ref TASK_REQUEST_ERRORS: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_tasks_request_errors",
            "Number of tasks request that failed"
        ),
        &["queue"]
    )
    .expect("Failed to create TASK_REQUEST_ERRORS counter");
    pub static ref TASK_REQUEST_TOTAL: CounterVec = CounterVec::new(
        Opts::new("replicore_tasks_request_total", "Number of tasks requested"),
        &["queue"]
    )
    .expect("Failed to create TASK_REQUEST_TOTAL counter");
    pub static ref TASK_WORKER_NO_HANDLER: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_tasks_worker_no_handler",
            "Number of tasks that failed because no handler was registered for their queue"
        ),
        &["queue"]
    )
    .expect("Failed to create TASK_WORKER_NO_HANDLER counter");
    pub static ref TASK_WORKER_POLL_ERRORS: Counter = Counter::new(
        "replicore_tasks_worker_poll_errors",
        "Number of tasks poll operation that failed"
    )
    .expect("Failed to create TASK_WORKER_POLL_ERRORS counter");
    pub static ref TASK_WORKER_RECEIVED: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_tasks_worker_received",
            "Number of tasks received from each queue"
        ),
        &["queue"]
    )
    .expect("Failed to create TASK_WORKER_RECEIVED counter");
}

/// Attemps to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    if let Err(err) = registry.register(Box::new(TASK_ACK_ERRORS.clone())) {
        debug!(logger, "Failed to register TASK_ACK_ERRORS"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(TASK_ACK_TOTAL.clone())) {
        debug!(logger, "Failed to register TASK_ACK_TOTAL"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(TASK_REQUEST_ERRORS.clone())) {
        debug!(logger, "Failed to register TASK_REQUEST_ERRORS"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(TASK_REQUEST_TOTAL.clone())) {
        debug!(logger, "Failed to register TASK_REQUEST_TOTAL"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(TASK_WORKER_NO_HANDLER.clone())) {
        debug!(logger, "Failed to register TASK_WORKER_NO_HANDLER"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(TASK_WORKER_POLL_ERRORS.clone())) {
        debug!(logger, "Failed to register TASK_WORKER_POLL_ERRORS"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(TASK_WORKER_RECEIVED.clone())) {
        debug!(logger, "Failed to register TASK_WORKER_RECEIVED"; "error" => ?err);
    }
}
