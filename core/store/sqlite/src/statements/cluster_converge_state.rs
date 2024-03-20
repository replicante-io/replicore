//! Persistent store operations on Cluster Converge State.
use anyhow::Result;
use opentelemetry_api::trace::FutureExt;
use tokio_rusqlite::Connection;

use replisdk::utils::metrics::CountFutureErrExt;
use replisdk::utils::trace::TraceFutureStdErrExt;

use replicore_cluster_models::ConvergeState;
use replicore_context::Context;
use replicore_store::ids::NamespacedResourceID;
use replicore_store::delete::DeleteClusterConvergeState;

const DELETE_SQL: &str = r#"
DELETE FROM store_cluster_converge_state
WHERE
    ns_id = ?1
    AND cluster_id = ?2
;"#;

const LOOKUP_SQL: &str = r#"
SELECT cluster_state
FROM store_cluster_converge_state
WHERE
    ns_id = ?1
    AND cluster_id = ?2
;"#;

const PERSIST_SQL: &str = r#"
INSERT INTO store_cluster_converge_state (ns_id, cluster_id, cluster_state)
VALUES (?1, ?2, ?3)
ON CONFLICT(ns_id, cluster_id)
DO UPDATE SET
    cluster_state=?3
;"#;

/// Delete a cluster convergence state from the store, ignoring missing clusters.
pub async fn delete(
    _: &Context,
    connection: &Connection,
    cluster: DeleteClusterConvergeState,
) -> Result<()> {
    let (err_count, _timer) = crate::telemetry::observe_op("clusterConvergeState.delete");
    let trace = crate::telemetry::trace_op("clusterConvergeState.delete");
    connection
        .call(move |connection| {
            connection.execute(
                DELETE_SQL,
                rusqlite::params![cluster.0.ns_id, cluster.0.name],
            )?;
            Ok(())
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await?;
    Ok(())
}

/// Lookup a cluster convergence state from the store, if one is available.
pub async fn lookup(
    _: &Context,
    connection: &Connection,
    cluster: NamespacedResourceID,
) -> Result<Option<ConvergeState>> {
    let (err_count, timer) = crate::telemetry::observe_op("clusterConvergeState.lookup");
    let trace = crate::telemetry::trace_op("clusterConvergeState.lookup");
    let cluster = connection
        .call(move |connection| {
            let mut statement = connection.prepare_cached(LOOKUP_SQL)?;
            let mut rows = statement.query([cluster.ns_id, cluster.name])?;
            let row = match rows.next()? {
                None => None,
                Some(row) => {
                    let cluster: String = row.get("cluster_state")?;
                    Some(cluster)
                }
            };
            Ok(row)
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await?;

    drop(timer);
    match cluster {
        None => Ok(None),
        Some(cluster) => {
            let cluster = replisdk::utils::encoding::decode_serde(&cluster)?;
            Ok(Some(cluster))
        }
    }
}

/// Persist a new or updated [`ConvergeState`]` into the store.
pub async fn persist(
    _: &Context,
    connection: &Connection,
    cluster: ConvergeState,
) -> Result<()> {
    let record = replisdk::utils::encoding::encode_serde(&cluster)?;
    let (err_count, _timer) = crate::telemetry::observe_op("clusterConvergeState.persist");
    let trace = crate::telemetry::trace_op("clusterConvergeState.persist");
    connection
        .call(move |connection| {
            connection.execute(
                PERSIST_SQL,
                rusqlite::params![cluster.ns_id, cluster.cluster_id, record],
            )?;
            Ok(())
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await?;
    Ok(())
}
