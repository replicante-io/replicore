use lazy_static::lazy_static;
use prometheus::CounterVec;
use prometheus::Opts;
use prometheus::Registry;
use slog::debug;
use slog::Logger;

lazy_static! {
    pub static ref ACK_ERROR: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_stream_ack_error",
            "Number of errors during message acknowledgement operations",
        ),
        &["stream", "group"]
    ).expect("Failed to create ACK_ERROR");
    pub static ref ACK_TOTAL: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_stream_acks",
            "Number of message acknowledgement operations",
        ),
        &["stream", "group"]
    ).expect("Failed to create ACK_TOTAL");
    pub static ref DELIVERED_ERROR: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_stream_delivery_error",
            "Number of errors during message delivery",
        ),
        &["stream", "group"]
    ).expect("Failed to create DELIVERED_ERROR");
    pub static ref DELIVERED_RETRY: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_stream_redelivered",
            "Number of messages redelivered",
        ),
        &["stream", "group"]
    ).expect("Failed to create DELIVERED_RETRY");
    pub static ref DELIVERED_TOTAL: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_stream_delivered",
            "Number of messages delivered (including redeliveries)",
        ),
        &["stream", "group"]
    ).expect("Failed to create DELIVERED_TOTAL");
    pub static ref EMIT_ERROR: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_stream_emit_error",
            "Number of messages errors while emitting messages",
        ),
        &["stream"]
    ).expect("Failed to create EMIT_ERROR");
    pub static ref EMIT_TOTAL: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_stream_emitted",
            "Number of messages emitted",
        ),
        &["stream"]
    ).expect("Failed to create EMIT_TOTAL");
    // TODO: exponential backoff
}

/// Attemps to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    if let Err(error) = registry.register(Box::new(ACK_ERROR.clone())) {
        debug!(logger, "Failed to register ACK_ERROR"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(ACK_TOTAL.clone())) {
        debug!(logger, "Failed to register ACK_TOTAL"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(DELIVERED_ERROR.clone())) {
        debug!(logger, "Failed to register DELIVERED_ERROR"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(DELIVERED_RETRY.clone())) {
        debug!(logger, "Failed to register DELIVERED_RETRY"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(DELIVERED_TOTAL.clone())) {
        debug!(logger, "Failed to register DELIVERED_TOTAL"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(EMIT_ERROR.clone())) {
        debug!(logger, "Failed to register EMIT_ERROR"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(EMIT_TOTAL.clone())) {
        debug!(logger, "Failed to register EMIT_TOTAL"; "error" => ?error);
    }
}
