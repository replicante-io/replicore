//! Persistent store operations on Orchestrator Actions.
use anyhow::Result;
use futures::StreamExt;
use opentelemetry_api::trace::FutureExt;
use tokio_rusqlite::Connection;

use replisdk::core::models::oaction::OAction;
use replisdk::utils::encoding;
use replisdk::utils::metrics::CountFutureErrExt;
use replisdk::utils::trace::TraceFutureStdErrExt;

use replicore_context::Context;
use replicore_store::query::ListOActions;
use replicore_store::query::LookupOAction;
use replicore_store::query::OActionEntryStream;

const LIST_ALL_SQL: &str = r#"
SELECT oaction
FROM store_oaction
WHERE
    ns_id = ?1
    AND cluster_id = ?2
ORDER BY created_ts ASC;
"#;

const LIST_UNFINISHED_SQL: &str = r#"
SELECT oaction
FROM store_oaction
WHERE
    ns_id = ?1
    AND cluster_id = ?2
    AND finished_ts IS NULL
ORDER BY created_ts ASC;
"#;

const LOOKUP_SQL: &str = r#"
SELECT oaction
FROM store_oaction
WHERE
    ns_id = ?1
    AND cluster_id = ?2
    AND action_id = ?3;
"#;

const PERSIST_SQL: &str = r#"
INSERT INTO store_oaction (
    ns_id,
    cluster_id,
    action_id,
    created_ts,
    finished_ts,
    oaction
)
VALUES (?1, ?2, ?3, ?4, ?5, ?6)
ON CONFLICT(ns_id, cluster_id, action_id)
DO UPDATE SET
    created_ts=?4,
    finished_ts=?5,
    oaction=?6
;"#;

/// Return a list of known [`OActionEntry`]s in the given cluster.
pub async fn list(
    _: &Context,
    connection: &Connection,
    query: ListOActions,
) -> Result<OActionEntryStream> {
    let (err_count, _timer) = crate::telemetry::observe_op("oaction.listEntries");
    let trace = crate::telemetry::trace_op("oaction.listEntries");
    let items = connection
        .call(move |connection| {
            let sql = if query.include_finished { LIST_ALL_SQL } else { LIST_UNFINISHED_SQL };
            let mut statement = connection.prepare_cached(sql)?;
            let mut rows = statement.query([query.ns_id, query.cluster_id])?;

            let mut items = Vec::new();
            while let Some(row) = rows.next()? {
                let item: String = row.get("oaction")?;
                items.push(item);
            }
            Ok(items)
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await?;

    let items = futures::stream::iter(items)
        .map(|oaction| {
            let oaction = encoding::decode_serde(&oaction)?;
            Ok(oaction)
        })
        .boxed();
    Ok(items)
}

/// Lookup an orchestrator action from the store, if one is available.
pub async fn lookup(
    _: &Context,
    connection: &Connection,
    query: LookupOAction,
) -> Result<Option<OAction>> {
    let (err_count, timer) = crate::telemetry::observe_op("oaction.lookup");
    let trace = crate::telemetry::trace_op("oaction.lookup");
    let action_id = query.0.action_id.to_string();
    let oaction = connection
        .call(move |connection| {
            let mut statement = connection.prepare_cached(LOOKUP_SQL)?;
            let mut rows = statement.query([
                query.0.ns_id,
                query.0.cluster_id,
                action_id,
            ])?;
            let row = match rows.next()? {
                None => None,
                Some(row) => {
                    let oaction: String = row.get("oaction")?;
                    Some(oaction)
                }
            };
            Ok(row)
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await?;

    drop(timer);
    match oaction {
        None => Ok(None),
        Some(oaction) => {
            let oaction = replisdk::utils::encoding::decode_serde(&oaction)?;
            Ok(Some(oaction))
        }
    }
}

/// Persist a new or updated record into the store.
pub async fn persist(_: &Context, connection: &Connection, oaction: OAction) -> Result<()> {
    // Serialise special types into stings for the DB.
    let created_ts = encoding::encode_time(oaction.created_ts)?;
    let finished_ts = encoding::encode_time_option_f64(oaction.finished_ts)?;

    // Execute the statement.
    let record = replisdk::utils::encoding::encode_serde(&oaction)?;
    let (err_count, _timer) = crate::telemetry::observe_op("oaction.persist");
    let trace = crate::telemetry::trace_op("oaction.persist");
    connection
        .call(move |connection| {
            connection.execute(PERSIST_SQL, rusqlite::params![
                oaction.ns_id,
                oaction.cluster_id,
                oaction.action_id.to_string(),
                created_ts,
                finished_ts,
                record,
            ])?;
            Ok(())
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await?;
    Ok(())
}
