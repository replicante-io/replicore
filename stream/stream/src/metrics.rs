use lazy_static::lazy_static;
use prometheus::CounterVec;
use prometheus::HistogramOpts;
use prometheus::HistogramVec;
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
    )
    .expect("Failed to create ACK_ERROR");
    pub static ref ACK_TOTAL: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_stream_acks",
            "Number of message acknowledgement operations",
        ),
        &["stream", "group"]
    )
    .expect("Failed to create ACK_TOTAL");
    pub static ref BACKOFF_DURATION: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "replicore_stream_backoff_duration",
            "Time spent in backed off waits for retry"
        ),
        &["stream", "group"]
    )
    .expect("Failed to create BACKOFF_DURATION");
    pub static ref BACKOFF_REPEAT: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_stream_backoff_repeat",
            "Number of times messages are backed off more then once",
        ),
        &["stream", "group"]
    )
    .expect("Failed to create BACKOFF_REPEAT");
    pub static ref BACKOFF_REQUIRED: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "replicore_stream_backoff_required",
            "Histogram of backoff retries required to process a message"
        )
        .buckets(vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]),
        &["stream", "group"]
    )
    .expect("Failed to create BACKOFF_REQUIRED");
    pub static ref BACKOFF_TOTAL: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_stream_backoff",
            "Number of times messages are backed off before retry",
        ),
        &["stream", "group"]
    )
    .expect("Failed to create BACKOFF_TOTAL");
    pub static ref DELIVERED_ERROR: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_stream_delivery_error",
            "Number of errors during message delivery",
        ),
        &["stream", "group"]
    )
    .expect("Failed to create DELIVERED_ERROR");
    pub static ref DELIVERED_RETRY: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_stream_redelivered",
            "Number of messages redelivered",
        ),
        &["stream", "group"]
    )
    .expect("Failed to create DELIVERED_RETRY");
    pub static ref DELIVERED_TOTAL: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_stream_delivered",
            "Number of messages delivered (including redeliveries)",
        ),
        &["stream", "group"]
    )
    .expect("Failed to create DELIVERED_TOTAL");
    pub static ref EMIT_ERROR: CounterVec = CounterVec::new(
        Opts::new(
            "replicore_stream_emit_error",
            "Number of messages errors while emitting messages",
        ),
        &["stream"]
    )
    .expect("Failed to create EMIT_ERROR");
    pub static ref EMIT_TOTAL: CounterVec = CounterVec::new(
        Opts::new("replicore_stream_emitted", "Number of messages emitted"),
        &["stream"]
    )
    .expect("Failed to create EMIT_TOTAL");
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
    if let Err(error) = registry.register(Box::new(BACKOFF_DURATION.clone())) {
        debug!(logger, "Failed to register BACKOFF_DURATION"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(BACKOFF_REPEAT.clone())) {
        debug!(logger, "Failed to register BACKOFF_REPEAT"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(BACKOFF_REQUIRED.clone())) {
        debug!(logger, "Failed to register BACKOFF_REQUIRED"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(BACKOFF_TOTAL.clone())) {
        debug!(logger, "Failed to register BACKOFF_TOTAL"; "error" => ?error);
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
