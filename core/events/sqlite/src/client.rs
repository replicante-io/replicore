//! SQLite async client.
use anyhow::Result;
use tokio_rusqlite::Connection;

use replicore_context::Context;

/// Special path requesting the use of an in-memory store.
pub const MEMORY_PATH: &str = ":memory:";

/// Name of the table to store refinery migration metadata into.
pub const REFINERY_SCHEMA_TABLE_NAME: &str = "refinery_schema_history__events";

/// Create a SQLite DB [`Connection`] to store events into.
///
/// The special [`MEMORY_PATH`] constant can be specified to create an in-memory store.
///
/// NOTE:
///   The use of an in-memory store is only intended for tests and experimentation
///   as all data will be lost as soon as the process terminates.
pub async fn create(context: &Context, path: &str) -> Result<Connection> {
    // Open or create the SQLite DB.
    let connection = if path == MEMORY_PATH {
        slog::warn!(
            context.logger,
            "Using in-memory store means data will be lost once the process terminates"
        );
        Connection::open_in_memory().await
    } else {
        Connection::open(path).await
    };
    let connection = connection?;
    Ok(connection)
}
