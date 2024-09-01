//! Persistent store operations on Cluster Shards.
use anyhow::Result;
use futures::StreamExt;
use opentelemetry_api::trace::FutureExt;
use tokio_rusqlite::Connection;

use replisdk::core::models::node::Shard;
use replisdk::utils::metrics::CountFutureErrExt;
use replisdk::utils::trace::TraceFutureStdErrExt;

use replicore_context::Context;
use replicore_store::ids::NodeID;
use replicore_store::query::ListShards;
use replicore_store::query::ShardsStream;

const DELETE_SQL: &str = r#"
DELETE FROM store_cluster_shard
WHERE
    ns_id = ?1
    AND cluster_id = ?2
    AND node_id = ?3
;"#;

const LIST_SQL: &str = r#"
SELECT shard
FROM store_cluster_shard
WHERE
    ns_id = ?1
    AND cluster_id = ?2
ORDER BY shard_id ASC;
"#;

const LIST_FOR_NODE_SQL: &str = r#"
SELECT shard
FROM store_cluster_shard
WHERE
    ns_id = ?1
    AND cluster_id = ?2
    AND node_id = ?3
ORDER BY shard_id ASC;
"#;

const PERSIST_SQL: &str = r#"
INSERT INTO store_cluster_shard (ns_id, cluster_id, node_id, shard_id, shard)
VALUES (?1, ?2, ?3, ?4, ?5)
ON CONFLICT(ns_id, cluster_id, node_id, shard_id)
DO UPDATE SET
    shard=?5
;"#;

/// Delete all shards located on a node.
pub async fn delete_on_node(_: &Context, connection: &Connection, node: NodeID) -> Result<()> {
    let (err_count, _timer) = crate::telemetry::observe_op("shard.delete");
    let trace = crate::telemetry::trace_op("shard.delete");
    connection
        .call(move |connection| {
            connection.execute(
                DELETE_SQL,
                rusqlite::params![node.ns_id, node.cluster_id, node.node_id],
            )?;
            Ok(())
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await?;
    Ok(())
}

/// Return a list of known [`Shard`]s in the given cluster.
pub async fn list(_: &Context, connection: &Connection, query: ListShards) -> Result<ShardsStream> {
    let (err_count, _timer) = crate::telemetry::observe_op("shard.list");
    let trace = crate::telemetry::trace_op("shard.list");
    let shards = connection
        .call(move |connection| {
            let sql = match query.node_id.is_some() {
                true => LIST_FOR_NODE_SQL,
                false => LIST_SQL,
            };
            let mut statement = connection.prepare_cached(sql)?;
            let mut rows = match query.node_id {
                None => statement.query([query.ns_id, query.cluster_id])?,
                Some(node_id) => statement.query([query.ns_id, query.cluster_id, node_id])?,
            };

            let mut shards = Vec::new();
            while let Some(row) = rows.next()? {
                let shard: String = row.get("shard")?;
                shards.push(shard);
            }
            Ok(shards)
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await?;

    let shards = futures::stream::iter(shards)
        .map(|shard| {
            let shard = replisdk::utils::encoding::decode_serde(&shard)?;
            Ok(shard)
        })
        .boxed();
    Ok(shards)
}

/// Persist a new or updated record into the store.
pub async fn persist(_: &Context, connection: &Connection, shard: Shard) -> Result<()> {
    let record = replisdk::utils::encoding::encode_serde(&shard)?;
    let (err_count, _timer) = crate::telemetry::observe_op("shard.persist");
    let trace = crate::telemetry::trace_op("shard.persist");
    connection
        .call(move |connection| {
            connection.execute(
                PERSIST_SQL,
                rusqlite::params![
                    shard.ns_id,
                    shard.cluster_id,
                    shard.node_id,
                    shard.shard_id,
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
