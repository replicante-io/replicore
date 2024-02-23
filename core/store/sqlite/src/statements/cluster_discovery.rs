//! Persistent store operations on Cluster Discoveries.
use anyhow::Result;
use opentelemetry_api::trace::FutureExt;
use tokio_rusqlite::Connection;

use replisdk::core::models::cluster::ClusterDiscovery;
use replisdk::utils::metrics::CountFutureErrExt;
use replisdk::utils::trace::TraceFutureStdErrExt;

use replicore_context::Context;
use replicore_store::ids::NamespacedResourceID;

const LOOKUP_SQL: &str = r#"
SELECT cluster_disc
FROM store_cluster_disc
WHERE
    ns_id = ?1
    AND cluster_id = ?2
;"#;

const PERSIST_SQL: &str = r#"
INSERT INTO store_cluster_disc (ns_id, cluster_id, cluster_disc)
VALUES (?1, ?2, ?3)
ON CONFLICT(ns_id, cluster_id)
DO UPDATE SET
    cluster_disc=?3
;"#;

/// Lookup a cluster discovery from the store, if one is available.
pub async fn lookup(
    _: &Context,
    connection: &Connection,
    cluster: NamespacedResourceID,
) -> Result<Option<ClusterDiscovery>> {
    let (err_count, timer) = crate::telemetry::observe_op("clusterDiscovery.lookup");
    let trace = crate::telemetry::trace_op("clusterDiscovery.lookup");
    let cluster = connection
        .call(move |connection| {
            let mut statement = connection.prepare_cached(LOOKUP_SQL)?;
            let mut rows = statement.query([cluster.ns_id, cluster.name])?;
            let row = match rows.next()? {
                None => None,
                Some(row) => {
                    let cluster: String = row.get("cluster_disc")?;
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

/// Persist a new or updated [`ClusterDiscovery`]` into the store.
pub async fn persist(
    _: &Context,
    connection: &Connection,
    cluster: ClusterDiscovery,
) -> Result<()> {
    let record = replisdk::utils::encoding::encode_serde(&cluster)?;
    let (err_count, _timer) = crate::telemetry::observe_op("clusterDiscovery.persist");
    let trace = crate::telemetry::trace_op("clusterDiscovery.persist");
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
