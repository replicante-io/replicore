//! Background Tasks operations to poll and acknowledge pending tasks.
use std::collections::HashMap;

use anyhow::Result;
use opentelemetry_api::trace::FutureExt;
use opentelemetry_api::Context as OTelContext;
use tokio_rusqlite::Connection;

use replisdk::utils::metrics::CountFutureErrExt;
use replisdk::utils::trace::TraceFutureStdErrExt;

use replicore_context::Context;
use replicore_tasks::conf::Queue;
use replicore_tasks::execute::ReceivedTask;

const DELETE_SQL: &str = r#"
DELETE FROM tasks_queue
WHERE task_id = ?1;
"#;

const GET_NEXT_SQL: &str = r#"
UPDATE tasks_queue
SET
    retries = retries - 1,
    next_retry = unixepoch() + retry_delay
WHERE task_id IN (
    SELECT task_id FROM tasks_queue
    WHERE
        queue_id IN ({%IDS%}) AND
        retries >= 0 AND
        (next_retry IS NULL OR next_retry <= unixepoch())
    ORDER BY task_id ASC
    LIMIT 1
)
RETURNING
    task_id,
    queue_id,
    payload,
    run_as,
    trace
;"#;

/// SQL extracted task object return from SQLite connection calls.
#[derive(Debug)]
pub struct SQLReceivedTask {
    task_id: String,
    queue_id: String,
    payload: String,
    run_as: Option<String>,
    trace: Option<String>,
}

pub async fn done(_: &Context, connection: &Connection, task: &ReceivedTask) -> Result<()> {
    let task_id = task.id.clone();
    let (err_count, _timer) = crate::telemetry::observe_op("task.next");
    let trace = crate::telemetry::trace_op("task.next");
    connection
        .call(move |connection| {
            connection.execute(DELETE_SQL, rusqlite::params![task_id])?;
            Ok(())
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await?;
    Ok(())
}

pub async fn next(
    _: &Context,
    connection: &Connection,
    queues: &HashMap<&'static String, &'static Queue>,
) -> Result<Option<ReceivedTask>> {
    // Can't easily pass a list of queues as a SQL parameter so we manually build the SQL.
    let subscribed = queues
        .keys()
        .map(|queue| format!("\"{}\"", queue))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = GET_NEXT_SQL.replace("{%IDS%}", &subscribed);

    // Query the next pending task using an update & return statement to avoid race conditions.
    let (err_count, timer) = crate::telemetry::observe_op("task.next");
    let trace = crate::telemetry::trace_op("task.next");
    let task = connection
        .call(move |connection| {
            let mut statement = connection.prepare_cached(&sql)?;
            let mut rows = statement.query([])?;
            let row = match rows.next()? {
                None => return Ok(None),
                Some(row) => row,
            };
            let task = SQLReceivedTask {
                task_id: row.get::<&str, i64>("task_id")?.to_string(),
                queue_id: row.get("queue_id")?,
                payload: row.get("payload")?,
                run_as: row.get("run_as")?,
                trace: row.get("trace")?,
            };
            Ok(Some(task))
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await?;
    drop(timer);
    let task = match task {
        None => return Ok(None),
        Some(task) => task,
    };

    let queue = queues
        .get(&task.queue_id)
        .expect("received task on unsubscribed queue");
    let payload = replisdk::utils::encoding::decode_serde(&task.payload)?;
    let run_as = replisdk::utils::encoding::decode_serde_option(&task.run_as)?;
    let trace = task.trace.map(decode_trace).transpose()?;
    let received = ReceivedTask {
        id: task.task_id,
        payload,
        queue,
        run_as,
        trace,
    };
    Ok(Some(received))
}

/// Extract an OpenTelemetry context from the encoded task data.
fn decode_trace(trace: String) -> Result<OTelContext> {
    let trace: HashMap<String, String> = replisdk::utils::encoding::decode_serde(&trace)?;
    let context =
        opentelemetry_api::global::get_text_map_propagator(|propagator| propagator.extract(&trace));
    Ok(context)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use once_cell::sync::Lazy;

    use replicore_tasks::conf::Queue;
    use replicore_tasks::execute::TaskAck;
    use replicore_tasks::execute::TaskSource;
    use replicore_tasks::execute::TEST_QUEUE_ALTERNATE;

    const NEXT_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(20);

    /// Fixed queue to submit unit test tasks to or receive them from.
    static EMPTY_QUEUE: Lazy<Queue> = Lazy::new(|| Queue {
        queue: String::from("UNIT_TEST_EMPTY"),
        retry_count: 2,
        retry_timeout: Duration::from_millis(50),
    });

    /// Insert tasks on different queues for tests.
    async fn insert_tasks(connection: &tokio_rusqlite::Connection) {
        connection
            .call(|connection| {
                connection
                    .execute(
                        r#"
                        INSERT INTO tasks_queue (queue_id, payload, retries, retry_delay)
                        VALUES
                            ("UNIT_TEST", "null", 0, 1),
                            ("UNIT_TEST_ALTERNATE", "null", -1, 30),
                            ("UNIT_TEST_ALTERNATE", "null", 5, 0),
                            ("UNIT_TEST_ALTERNATE", "null", 0, 3)
                        ;
                        "#,
                        rusqlite::params![],
                    )
                    .unwrap();
                Ok(())
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn next_task() {
        let backend = crate::statements::tests::sqlite_tasks().await;
        let connection = backend.connection.clone();
        let context = replicore_context::Context::fixture();
        let mut source = TaskSource::from(backend);
        insert_tasks(&connection).await;

        // Fetch the next task and check it is what we expect.
        source
            .subscribe(&context, &TEST_QUEUE_ALTERNATE)
            .await
            .unwrap();
        let task = tokio::time::timeout(NEXT_TIMEOUT, source.next(&context))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(task.id, "3");
        assert_eq!(task.queue.queue, "UNIT_TEST_ALTERNATE");

        // Check the DB is updated as needed.
        connection
            .call(|connection| {
                let mut statement =
                    connection.prepare_cached("SELECT * FROM tasks_queue WHERE task_id = ?1;")?;
                let mut rows = statement.query(["3"])?;
                let row = rows.next()?.unwrap();
                let retries: i64 = row.get("retries").unwrap();
                let next_retry: Option<i64> = row.get("next_retry").unwrap();

                assert_eq!(retries, 4);
                assert!(next_retry.is_some());
                assert!(next_retry.unwrap() > 0);
                Ok(())
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn next_task_retry() {
        let backend = crate::statements::tests::sqlite_tasks().await;
        let connection = backend.connection.clone();
        let context = replicore_context::Context::fixture();
        let mut source = TaskSource::from(backend);
        insert_tasks(&connection).await;

        // Fetch the next task and check it is what we expect.
        source
            .subscribe(&context, &TEST_QUEUE_ALTERNATE)
            .await
            .unwrap();
        let task = tokio::time::timeout(NEXT_TIMEOUT, source.next(&context))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(task.id, "3");
        assert_eq!(task.queue.queue, "UNIT_TEST_ALTERNATE");

        // Fetch the next task after a retry delay and check it is the same.
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let task = tokio::time::timeout(NEXT_TIMEOUT, source.next(&context))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(task.id, "3");
        assert_eq!(task.queue.queue, "UNIT_TEST_ALTERNATE");
    }

    #[tokio::test]
    async fn next_task_waits() {
        let backend = crate::statements::tests::sqlite_tasks().await;
        let connection = backend.connection.clone();
        let context = replicore_context::Context::fixture();
        let mut source = TaskSource::from(backend);
        insert_tasks(&connection).await;

        // Try to fetch a task and expect a timeout from tokio.
        source.subscribe(&context, &EMPTY_QUEUE).await.unwrap();
        let task = tokio::time::timeout(NEXT_TIMEOUT, source.next(&context)).await;
        assert!(task.is_err());
    }

    #[tokio::test]
    async fn remove_task_on_done() {
        let backend = crate::statements::tests::sqlite_tasks().await;
        let connection = backend.connection.clone();
        let context = replicore_context::Context::fixture();
        let ack = TaskAck::from(backend.clone());
        let mut source = TaskSource::from(backend);
        insert_tasks(&connection).await;

        // Fetch the next task and acknowledge it as done.
        source
            .subscribe(&context, &TEST_QUEUE_ALTERNATE)
            .await
            .unwrap();
        let task = tokio::time::timeout(NEXT_TIMEOUT, source.next(&context))
            .await
            .unwrap()
            .unwrap();
        let task_id = task.id.clone();
        ack.done(&context, &task).await.unwrap();

        // Check the DB to make sure the task is deleted.
        connection
            .call(|connection| {
                let mut statement =
                    connection.prepare_cached("SELECT * FROM tasks_queue WHERE task_id = ?1;")?;
                let mut rows = statement.query([task_id])?;
                let row = rows.next()?;
                assert!(row.is_none());
                Ok(())
            })
            .await
            .unwrap();
    }
}
