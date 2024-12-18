//! Telemetry related to tasks execution and submission.
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use anyhow::Result;
use once_cell::sync::Lazy;
use opentelemetry::global::BoxedTracer;
use prometheus::Counter;
use prometheus::CounterVec;
use prometheus::Opts;

/// Total number of task received for execution.
pub static RECEIVE_COUNT: Lazy<Counter> = Lazy::new(|| {
    Counter::new(
        "replicore_tasks_receive_count",
        "Total number of task received for execution",
    )
    .expect("failed to initialise RECEIVE_COUNT counter")
});

/// Number of task receive operations that resulted in error.
pub static RECEIVE_ERR: Lazy<Counter> = Lazy::new(|| {
    Counter::new(
        "replicore_tasks_receive_error",
        "Number of task receive operations that resulted in error",
    )
    .expect("failed to initialise RECEIVE_ERR counter")
});

/// Total number of task submissions.
pub static SUBMIT_COUNT: Lazy<CounterVec> = Lazy::new(|| {
    CounterVec::new(
        Opts::new(
            "replicore_tasks_submit_count",
            "Total number of task submissions",
        ),
        &["queue"],
    )
    .expect("failed to initialise SUBMIT_COUNT counter")
});

/// Number of task submissions that resulted in error.
pub static SUBMIT_ERR: Lazy<CounterVec> = Lazy::new(|| {
    CounterVec::new(
        Opts::new(
            "replicore_tasks_submit_error",
            "Number of task submissions that resulted in error",
        ),
        &["queue"],
    )
    .expect("failed to initialise SUBMIT_ERR counter")
});

/// Open Telemetry tracer task operations.
pub static TRACER: Lazy<BoxedTracer> = Lazy::new(|| {
    let scope = opentelemetry::InstrumentationScope::builder(env!("CARGO_PKG_NAME"))
        .with_version(env!("CARGO_PKG_VERSION"))
        .build();
    opentelemetry::global::tracer_with_scope(scope)
});

/// Ensure metrics are registered only once.
static METRICS_REGISTERED: AtomicBool = AtomicBool::new(false);

/// The first time this method is called it will register the SQLite events backend metrics.
pub fn register_metrics(reg: &prometheus::Registry) -> Result<()> {
    // Skip registration if already done before.
    if METRICS_REGISTERED.swap(true, Ordering::AcqRel) {
        return Ok(());
    }

    let collectors: [Box<dyn prometheus::core::Collector>; 4] = [
        Box::new(RECEIVE_COUNT.clone()),
        Box::new(RECEIVE_ERR.clone()),
        Box::new(SUBMIT_COUNT.clone()),
        Box::new(SUBMIT_ERR.clone()),
    ];
    for collector in collectors {
        reg.register(collector)?;
    }
    Ok(())
}
