//! Background Tasks operations to submit tasks to the queue.
use anyhow::Result;
use opentelemetry_api::trace::FutureExt;
use tokio_rusqlite::Connection;

use replisdk::utils::metrics::CountFutureErrExt;
use replisdk::utils::trace::TraceFutureStdErrExt;

use replicore_context::Context;
use replicore_tasks::submit::TaskSubmission;

const SUBMIT_SQL: &str = r#"
INSERT INTO tasks_queue (queue_id, payload, run_as, trace, retries, retry_delay)
VALUES (?1, ?2, ?3, ?4, ?5, ?6);
"#;

pub async fn submit(_: &Context, connection: &Connection, task: TaskSubmission) -> Result<()> {
    let payload = replisdk::utils::encoding::encode_serde(&task.payload)?;
    let run_as = replisdk::utils::encoding::encode_serde_option(&task.run_as)?;
    let trace_context = task
        .trace
        .map(|trace| {
            opentelemetry_api::global::get_text_map_propagator(|propagator| {
                let mut buffer = std::collections::HashMap::new();
                propagator.inject_context(&trace, &mut buffer);
                serde_json::to_string(&buffer)
            })
        })
        .transpose()?;
    let queue_id = &task.queue.queue;
    let retries = task.queue.retry_count;
    let retry_delay = task.queue.retry_timeout.as_secs();
    let (err_count, _timer) = crate::telemetry::observe_op("task.submit");
    let trace = crate::telemetry::trace_op("task.submit");
    connection
        .call(move |connection| {
            connection.execute(
                SUBMIT_SQL,
                rusqlite::params![queue_id, payload, run_as, trace_context, retries, retry_delay],
            )?;
            Ok(())
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use replicore_tasks::execute::TEST_QUEUE;
    use replicore_tasks::submit::TaskSubmission;
    use replicore_tasks::submit::Tasks;

    #[tokio::test]
    async fn submit() {
        let backend = crate::statements::tests::sqlite_tasks().await;
        let connection = backend.connection.clone();
        let context = replicore_context::Context::fixture();
        let task = TaskSubmission::new(&TEST_QUEUE, &false).unwrap();
        let tasks = Tasks::from(backend);
        tasks.submit(&context, task).await.unwrap();

        let count = connection
            .call(move |connection| {
                let mut statement =
                    connection.prepare_cached("SELECT COUNT(*) FROM tasks_queue;")?;
                let mut rows = statement.query([])?;
                let row = rows.next()?.expect("count of table records");
                let count: u64 = row.get("COUNT(*)")?;
                Ok(count)
            })
            .await
            .expect("count SQL execution error");
        assert_eq!(count, 1);
    }
}
