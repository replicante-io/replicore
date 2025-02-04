//! SQL statements to implement the [`ILease`] backend with SQLite.
use std::time::Duration;

use anyhow::Result;
use opentelemetry::trace::FutureExt;
use time::OffsetDateTime;
use tokio_rusqlite::Connection;

use replisdk::utils::metrics::CountFutureErrExt;
use replisdk::utils::trace::TraceFutureStdErrExt;

use replicore_context::Context;

const ACQUIRE_SQL: &str = r#"
INSERT INTO coordinator_lease (lease_id, owner_value, last_renew, expired_after)
VALUES (?1, ?2, ?3, ?4)
ON CONFLICT(lease_id)
    DO UPDATE SET
        owner_value = ?2,
        last_renew = ?3,
        expired_after = ?4
    WHERE coordinator_lease.expired_after < unixepoch()
RETURNING owner_value, expired_after
;"#;

const CHECK_SQL: &str = r#"
SELECT 1
FROM coordinator_lease
WHERE
    lease_id = ?1
    AND unixepoch() < coordinator_lease.expired_after
;"#;

/// Maintenance SQL to delete records for expired leases.
///
/// For safety:
///
/// - Delete leases only if they expired at least 30 seconds ago.
/// - Limit the amount of rows deleted at once to mitigate performance problems.
const MAINTENANCE_SQL: &str = r#"
DELETE FROM coordinator_lease
WHERE (coordinator_lease.expired_after + 30) < unixepoch()
;"#;

const RENEW_SQL: &str = r#"
UPDATE coordinator_lease
SET
    last_renew = ?3,
    expired_after = ?4
WHERE
    lease_id = ?1
    AND owner_value = ?2
    AND unixepoch() < coordinator_lease.expired_after
;"#;

const STEP_DOWN_SQL: &str = r#"
DELETE FROM coordinator_lease
WHERE
    lease_id = ?1
    AND owner_value = ?2
;"#;

/// Attempt to acquire a new lease.
pub async fn acquire(
    _: &Context,
    connection: &Connection,
    lease_id: String,
    owner_value: String,
    ttl: Duration,
) -> Result<String> {
    let last_renew = OffsetDateTime::now_utc();
    let expired_after = last_renew + ttl;

    // Attempt to acquire the lease in the DB.
    let (err_count, _timer) = crate::telemetry::observe_op("lease.acquire");
    let trace = crate::telemetry::trace_op("lease.acquire");
    let owner_value = connection
        .call(move |connection| {
            let last_renew = last_renew.unix_timestamp();
            let expired_after = expired_after.unix_timestamp();
            let mut statement = connection.prepare_cached(ACQUIRE_SQL)?;
            let mut rows = statement.query(rusqlite::params![
                lease_id,
                owner_value,
                last_renew,
                expired_after
            ])?;

            // Check what is stored in the DB.
            let row = match rows.next()? {
                None => return Ok(None),
                Some(row) => row,
            };
            let owner_value: String = row.get("owner_value")?;
            Ok(Some(owner_value))
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await?;

    // The acquire SQL does not return a value if the lease is already owned.
    Ok(owner_value.unwrap_or_default())
}

/// Check if a lease is expired or missing from the DB.
pub async fn expired_or_lost(
    _: &Context,
    connection: &Connection,
    lease_id: String,
) -> Result<bool> {
    // Check if the lease record is missing or expired.
    let (err_count, _timer) = crate::telemetry::observe_op("lease.expiredOrLost");
    let trace = crate::telemetry::trace_op("lease.expiredOrLost");
    connection
        .call(move |connection| {
            let mut statement = connection.prepare_cached(CHECK_SQL)?;
            let mut rows = statement.query(rusqlite::params![lease_id])?;
            let row = rows.next()?;
            Ok(row.is_none())
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await
        .map_err(anyhow::Error::from)
}

/// Perform maintenance tasks on the SQLite database.
///
/// The maintenance procedure cleans up lease records for expired leases.
/// This helps the DB remain clean and forget leases lost long ago.
pub async fn maintenance(context: &Context, connection: &Connection) -> Result<()> {
    // Execute maintenance SQL statement.
    let (err_count, timer) = crate::telemetry::observe_op("lease.maintenance");
    let trace = crate::telemetry::trace_op("lease.maintenance");
    let count = connection
        .call(move |connection| {
            let count = connection.execute(MAINTENANCE_SQL, rusqlite::params![])?;
            Ok(count)
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await
        .map_err(anyhow::Error::from)?;
    drop(timer);

    if count > 0 {
        slog::info!(
            context.logger,
            "SQLite coordinator maintenance deleted {} expired leases",
            count
        );
    }
    Ok(())
}

/// Renew a lease that is currently held.
///
/// If the lease has been lost (renew after TTL or record missing) the function returns `false`.
pub async fn renew_primary(
    _: &Context,
    connection: &Connection,
    lease_id: String,
    owner_value: String,
    ttl: Duration,
) -> Result<bool> {
    let last_renew = OffsetDateTime::now_utc();
    let expired_after = last_renew + ttl;

    // Attempt to renew the lease in the DB.
    let (err_count, _timer) = crate::telemetry::observe_op("lease.renewPrimary");
    let trace = crate::telemetry::trace_op("lease.renewPrimary");
    connection
        .call(move |connection| {
            let last_renew = last_renew.unix_timestamp();
            let expired_after = expired_after.unix_timestamp();
            let count = connection.execute(
                RENEW_SQL,
                rusqlite::params![lease_id, owner_value, last_renew, expired_after],
            )?;
            assert!(count <= 1, "Lease renewal applied to more then one record");
            Ok(count == 1)
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await
        .map_err(anyhow::Error::from)
}

/// Step down by deleting the lease record, if we are primary.
///
/// If the lease does not exist or the owner value does not match returns `false`.
pub async fn step_down(
    context: &Context,
    connection: &Connection,
    lease_id: String,
    owner_value: String,
) -> Result<bool> {
    slog::debug!(context.logger, "SQLite lease step down"; "lease-id" => &lease_id);

    // Attempt to remove the lease record from the DB.
    let (err_count, _timer) = crate::telemetry::observe_op("lease.stepDown");
    let trace = crate::telemetry::trace_op("lease.stepDown");
    connection
        .call(move |connection| {
            let count =
                connection.execute(STEP_DOWN_SQL, rusqlite::params![lease_id, owner_value])?;
            assert!(count <= 1, "Lease step down deleted more then one record");
            Ok(count == 1)
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await
        .map_err(anyhow::Error::from)
}
