//! Telemetry related to the SQLite backed tasks implementation.
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use anyhow::Result;
use once_cell::sync::Lazy;
use opentelemetry_api::global::BoxedTracer;
use opentelemetry_api::trace::SpanKind;
use opentelemetry_api::trace::TraceContextExt;
use opentelemetry_api::trace::Tracer;
use opentelemetry_api::trace::TracerProvider;
use opentelemetry_api::Context;
use prometheus::Counter;
use prometheus::CounterVec;
use prometheus::HistogramOpts;
use prometheus::HistogramTimer;
use prometheus::HistogramVec;
use prometheus::Opts;

/// Duration (in seconds) of SQLite operations.
pub static OPS_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    HistogramVec::new(
        HistogramOpts::new(
            "replicore_tasks_sqlite_ops_duration",
            "Duration (in seconds) of SQLite operations",
        )
        .buckets(vec![0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
        &["op"],
    )
    .expect("failed to initialise OPS_DURATION histogram")
});

/// Number of SQLite operations that resulted in error.
pub static OPS_ERR: Lazy<CounterVec> = Lazy::new(|| {
    CounterVec::new(
        Opts::new(
            "replicore_tasks_sqlite_ops_error",
            "Number of SQLite operations that resulted in error",
        ),
        &["op"],
    )
    .expect("failed to initialise OPS_ERR counter")
});

/// Open Telemetry tracer for the SQLite events backend.
pub static TRACER: Lazy<BoxedTracer> = Lazy::new(|| {
    opentelemetry_api::global::tracer_provider().versioned_tracer(
        env!("CARGO_PKG_NAME"),
        Some(env!("CARGO_PKG_VERSION")),
        Option::<&str>::None,
        None,
    )
});

/// Ensure metrics are registered only once.
static METRICS_REGISTERED: AtomicBool = AtomicBool::new(false);

/// The first time this method is called it will register the SQLite events backend metrics.
pub fn register_metrics(reg: &prometheus::Registry) -> Result<()> {
    // Skip registration if already done before.
    if METRICS_REGISTERED.swap(true, Ordering::AcqRel) {
        return Ok(());
    }

    let collectors: [Box<dyn prometheus::core::Collector>; 2] =
        [Box::new(OPS_DURATION.clone()), Box::new(OPS_ERR.clone())];
    for collector in collectors {
        reg.register(collector)?;
    }
    Ok(())
}

/// Observe the execution of an SQLite operation.
///
/// ## Returns
///
/// - A started timer to observe the duration of the operation.
/// - A [`Counter`] to increment in case of error.
#[inline]
pub fn observe_op(op: &str) -> (Counter, HistogramTimer) {
    let err_count = OPS_ERR.with_label_values(&[op]);
    let timer = OPS_DURATION.with_label_values(&[op]).start_timer();
    (err_count, timer)
}

/// Initialised a new span and context for Agent Store operations,
///
/// The new span and context are automatically children of the active span and context.
pub fn trace_op(op: &str) -> Context {
    let mut builder = TRACER.span_builder(op.to_string());
    builder.span_kind = Some(SpanKind::Client);
    let parent = Context::current();
    let span = TRACER.build_with_context(builder, &parent);
    parent.with_span(span)
}
