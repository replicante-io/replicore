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
use replicore_store::query::LookupNAction;
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

const LIST_SQL_START: &str = r#"
SELECT naction
FROM store_naction
WHERE
    ns_id = ?1
    AND cluster_id = ?2
"#;

const LIST_SQL_NODE: &str = r#"    AND node_id = ?3
"#;

const LIST_SQL_UNFINISHED: &str = r#"    AND finished_time IS NULL
"#;

const LIST_SQL_END: &str = r#"ORDER BY created_time ASC;"#;

const LOOKUP_SQL: &str = r#"
SELECT naction
FROM store_naction
WHERE
    ns_id = ?1
    AND cluster_id = ?2
    AND node_id = ?3
    AND action_id = ?4;
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
    // Build the full SQL statement including the correct filters.
    let sql = match (query.include_finished, query.node_id.is_some()) {
        (true, true) => format!("{}{}", LIST_SQL_NODE, LIST_SQL_UNFINISHED),
        (false, true) => LIST_SQL_NODE.to_string(),
        (false, false) => LIST_SQL_UNFINISHED.to_string(),
        _ => String::from(""),
    };
    let sql = format!("{}{}{}", LIST_SQL_START, sql, LIST_SQL_END);

    // Execute the select statement.
    let (err_count, _timer) = crate::telemetry::observe_op("naction.list");
    let trace = crate::telemetry::trace_op("naction.list");
    let items = connection
        .call(move |connection| {
            let mut statement = connection.prepare_cached(&sql)?;
            let mut rows = match query.node_id {
                None => statement.query([query.ns_id, query.cluster_id])?,
                Some(node_id) => statement.query([query.ns_id, query.cluster_id, node_id])?,
            };

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

/// Lookup a node action from the store, if one is available.
pub async fn lookup(
    _: &Context,
    connection: &Connection,
    query: LookupNAction,
) -> Result<Option<NAction>> {
    let (err_count, timer) = crate::telemetry::observe_op("naction.lookup");
    let trace = crate::telemetry::trace_op("naction.lookup");
    let action_id = query.0.action_id.to_string();
    let action = connection
        .call(move |connection| {
            let mut statement = connection.prepare_cached(LOOKUP_SQL)?;
            let mut rows = statement.query([
                query.0.ns_id,
                query.0.cluster_id,
                query.0.node_id,
                action_id,
            ])?;
            let row = match rows.next()? {
                None => None,
                Some(row) => {
                    let action: String = row.get("naction")?;
                    Some(action)
                }
            };
            Ok(row)
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await?;

    drop(timer);
    match action {
        None => Ok(None),
        Some(action) => {
            let action = replisdk::utils::encoding::decode_serde(&action)?;
            Ok(Some(action))
        }
    }
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
            let sql = format!("{}{}{}", LIST_SQL_START, LIST_SQL_UNFINISHED, LIST_SQL_END);
            let mut statement = connection.prepare_cached(&sql)?;
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
