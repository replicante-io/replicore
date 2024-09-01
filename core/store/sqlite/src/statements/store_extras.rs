//! Persistent store operations on Cluster Nodes.
use anyhow::Result;
use futures::StreamExt;
use opentelemetry_api::trace::FutureExt;
use tokio_rusqlite::Connection;

use replisdk::core::models::node::StoreExtras;
use replisdk::utils::metrics::CountFutureErrExt;
use replisdk::utils::trace::TraceFutureStdErrExt;

use replicore_context::Context;
use replicore_store::ids::NamespacedResourceID;
use replicore_store::ids::NodeID;
use replicore_store::query::StoreExtrasStream;

const DELETE_SQL: &str = r#"
DELETE FROM store_cluster_store_extras
WHERE
    ns_id = ?1
    AND cluster_id = ?2
    AND node_id = ?3
;"#;

const LIST_SQL: &str = r#"
SELECT store_extras
FROM store_cluster_store_extras
WHERE
    ns_id = ?1
    AND cluster_id = ?2
ORDER BY node_id ASC;
"#;

const PERSIST_SQL: &str = r#"
INSERT INTO store_cluster_store_extras (ns_id, cluster_id, node_id, store_extras)
VALUES (?1, ?2, ?3, ?4)
ON CONFLICT(ns_id, cluster_id, node_id)
DO UPDATE SET
    store_extras=?4
;"#;

/// Delete the [`StoreExtras`] record for a node.
pub async fn delete(_: &Context, connection: &Connection, node: NodeID) -> Result<()> {
    let (err_count, _timer) = crate::telemetry::observe_op("storeExtra.delete");
    let trace = crate::telemetry::trace_op("storeExtra.delete");
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

/// Return a list of known [`StoreExtras`]s in the given cluster.
pub async fn list(
    _: &Context,
    connection: &Connection,
    query: NamespacedResourceID,
) -> Result<StoreExtrasStream> {
    let (err_count, _timer) = crate::telemetry::observe_op("storeExtra.list");
    let trace = crate::telemetry::trace_op("storeExtras.list");
    let nodes = connection
        .call(move |connection| {
            let mut statement = connection.prepare_cached(LIST_SQL)?;
            let mut rows = statement.query([query.ns_id, query.name])?;

            let mut nodes = Vec::new();
            while let Some(row) = rows.next()? {
                let node: String = row.get("store_extras")?;
                nodes.push(node);
            }
            Ok(nodes)
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await?;

    let nodes = futures::stream::iter(nodes)
        .map(|node| {
            let node = replisdk::utils::encoding::decode_serde(&node)?;
            Ok(node)
        })
        .boxed();
    Ok(nodes)
}

/// Persist a new or updated record into the store.
pub async fn persist(_: &Context, connection: &Connection, extras: StoreExtras) -> Result<()> {
    let record = replisdk::utils::encoding::encode_serde(&extras)?;
    let (err_count, _timer) = crate::telemetry::observe_op("storeExtras.persist");
    let trace = crate::telemetry::trace_op("storeExtras.persist");
    connection
        .call(move |connection| {
            connection.execute(
                PERSIST_SQL,
                rusqlite::params![extras.ns_id, extras.cluster_id, extras.node_id, record],
            )?;
            Ok(())
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await?;
    Ok(())
}
