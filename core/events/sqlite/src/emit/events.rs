//! Emit events to the SQLite store.
use anyhow::Result;
use opentelemetry_api::trace::FutureExt;
use tokio_rusqlite::Connection;

use replisdk::utils::encoding;
use replisdk::utils::metrics::CountFutureErrExt;
use replisdk::utils::trace::TraceFutureStdErrExt;

use replicore_context::Context;
use replicore_events::emit::EventsBackend;
use replicore_events::Event;

use crate::Conf;

const EMIT_AUDIT_SQL: &str = r#"
INSERT INTO events_audit (event, time)
VALUES (?1, ?2);
"#;

const EMIT_CHANGE_SQL: &str = r#"
INSERT INTO events_change (event, time)
VALUES (?1, ?2);
"#;

/// SQLite backed events implementation/
pub struct SQLiteEvents {
    connection: Connection,
}

impl SQLiteEvents {
    /// Initialise the SQLite events backend.
    pub async fn new(context: &Context, conf: &Conf) -> Result<Self> {
        let connection = crate::client::create(context, &conf.path).await?;
        Ok(SQLiteEvents { connection })
    }
}

#[async_trait::async_trait]
impl EventsBackend for SQLiteEvents {
    async fn audit(&self, _: &Context, event: Event) -> Result<()> {
        // Serialise the event.
        let serialised = encoding::encode_serde(&event)?;
        let time = encoding::encode_time(event.time)?;

        // Insert it into the DB.
        let (err_count, _timer) = crate::telemetry::observe_op("emit.audit");
        let trace = crate::telemetry::trace_op("emit.audit");
        self.connection
            .call(move |connection| {
                connection.execute(EMIT_AUDIT_SQL, rusqlite::params![serialised, time])?;
                Ok(())
            })
            .count_on_err(err_count)
            .trace_on_err_with_status()
            .with_context(trace)
            .await?;
        Ok(())
    }

    async fn change(&self, _: &Context, event: Event) -> Result<()> {
        // Serialise the event.
        let serialised = encoding::encode_serde(&event)?;
        let time = encoding::encode_time(event.time)?;

        // Insert it into the DB.
        let (err_count, _timer) = crate::telemetry::observe_op("emit.change");
        let trace = crate::telemetry::trace_op("emit.change");
        self.connection
            .call(move |connection| {
                connection.execute(EMIT_CHANGE_SQL, rusqlite::params![serialised, time])?;
                Ok(())
            })
            .count_on_err(err_count)
            .trace_on_err_with_status()
            .with_context(trace)
            .await?;
        Ok(())
    }
}
