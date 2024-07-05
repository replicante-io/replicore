//! Persistent store operations on Node Actions.
use anyhow::Result;
use futures::StreamExt;
use opentelemetry_api::trace::FutureExt;
use tokio_rusqlite::Connection;

use replisdk::core::models::naction::NAction;
use replisdk::utils::encoding;
use replisdk::utils::metrics::CountFutureErrExt;
use replisdk::utils::trace::TraceFutureStdErrExt;

use replicore_context::Context;
use replicore_store::ids::NamespacedResourceID;
use replicore_store::query::ListNActions;
use replicore_store::query::NActionEntryStream;
use replicore_store::query::NActionStream;

const CANCEL_FOR_NODE_SQL: &str = r#"
UPDATE store_naction
SET
  naction = json_set(
    json_set(naction, '$.finished_time', ?1),
    '$.state.phase', "CANCELLED"
  ),
  finished_time = ?2
WHERE
  ns_id = ?3
  AND cluster_id = ?4,
  AND node_id = ?5
;"#;

const LIST_ALL_SQL: &str = r#"
SELECT naction
FROM store_naction
WHERE
    ns_id = ?1
    AND cluster_id = ?2
ORDER BY created_time ASC;
"#;

const LIST_UNFINISHED_SQL: &str = r#"
SELECT naction
FROM store_naction
WHERE
    ns_id = ?1
    AND cluster_id = ?2
    AND finished_time IS NULL
ORDER BY created_time ASC;
"#;

const PERSIST_SQL: &str = r#"
INSERT INTO store_naction (
    ns_id,
    cluster_id,
    node_id,
    action_id,
    created_time,
    finished_time,
    naction
)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
ON CONFLICT(ns_id, cluster_id, node_id, action_id)
DO UPDATE SET
    created_time=?5,
    finished_time=?6,
    naction=?7
;"#;

/// Cancel all actions for a given node.
pub async fn cancel_for_node(
    _: &Context,
    connection: &Connection,
    node: replicore_store::ids::NodeID,
) -> Result<()> {
    let finished_time = time::OffsetDateTime::now_utc();
    let finished_num = replisdk::utils::encoding::encode_time_f64(finished_time)?;
    let finished_str = replisdk::utils::encoding::encode_time(finished_time)?;

    let (err_count, _timer) = crate::telemetry::observe_op("naction.cancelForNode");
    let trace = crate::telemetry::trace_op("naction.cancelForNode");
    connection
        .call(move |connection| {
            connection.execute(
                CANCEL_FOR_NODE_SQL,
                rusqlite::params![
                    finished_str,
                    finished_num,
                    node.ns_id,
                    node.cluster_id,
                    node.node_id,
                ],
            )?;
            Ok(())
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await?;
    Ok(())
}

/// Return a list of known [`NActionEntry`]s in the given cluster.
pub async fn list(
    _: &Context,
    connection: &Connection,
    query: ListNActions,
) -> Result<NActionEntryStream> {
    let (err_count, _timer) = crate::telemetry::observe_op("naction.list");
    let trace = crate::telemetry::trace_op("naction.list");
    let items = connection
        .call(move |connection| {
            let sql = match query.include_finished {
                true => LIST_ALL_SQL,
                false => LIST_UNFINISHED_SQL,
            };
            let mut statement = connection.prepare_cached(sql)?;
            let mut rows = statement.query([query.ns_id, query.cluster_id])?;

            let mut items = Vec::new();
            while let Some(row) = rows.next()? {
                let item: String = row.get("naction")?;
                items.push(item);
            }
            Ok(items)
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await?;

    let items = futures::stream::iter(items)
        .map(|action| {
            let action = encoding::decode_serde(&action)?;
            Ok(action)
        })
        .boxed();
    Ok(items)
}

/// Persist a new or updated record into the store.
pub async fn persist(_: &Context, connection: &Connection, action: NAction) -> Result<()> {
    // Serialise special types into stings for the DB.
    let created_time = encoding::encode_time(action.created_time)?;
    let finished_time = encoding::encode_time_option_f64(action.finished_time)?;

    // Execute the statement.
    let record = replisdk::utils::encoding::encode_serde(&action)?;
    let (err_count, _timer) = crate::telemetry::observe_op("naction.persist");
    let trace = crate::telemetry::trace_op("naction.persist");
    connection
        .call(move |connection| {
            connection.execute(
                PERSIST_SQL,
                rusqlite::params![
                    action.ns_id,
                    action.cluster_id,
                    action.node_id,
                    action.action_id.to_string(),
                    created_time,
                    finished_time,
                    record,
                ],
            )?;
            Ok(())
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await?;
    Ok(())
}

/// Iterate over unfinished node actions.
pub async fn unfinished(
    _: &Context,
    connection: &Connection,
    query: NamespacedResourceID,
) -> Result<NActionStream> {
    let (err_count, _timer) = crate::telemetry::observe_op("naction.unfinished");
    let trace = crate::telemetry::trace_op("naction.unfinished");
    let items = connection
        .call(move |connection| {
            let mut statement = connection.prepare_cached(LIST_UNFINISHED_SQL)?;
            let mut rows = statement.query([query.ns_id, query.name])?;

            let mut items = Vec::new();
            while let Some(row) = rows.next()? {
                let item: String = row.get("naction")?;
                items.push(item);
            }
            Ok(items)
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await?;

    let items = futures::stream::iter(items)
        .map(|action| {
            let action = encoding::decode_serde(&action)?;
            Ok(action)
        })
        .boxed();
    Ok(items)
}
