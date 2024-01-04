//! Persistent store operations on Cluster Specifications.
use anyhow::Result;
use futures::StreamExt;
use opentelemetry_api::trace::FutureExt;
use tokio_rusqlite::Connection;

use replisdk::core::models::cluster::ClusterSpec;
use replisdk::utils::metrics::CountFutureErrExt;
use replisdk::utils::trace::TraceFutureStdErrExt;

use replicore_context::Context;
use replicore_store::delete::DeleteClusterSpec;
use replicore_store::ids::NamespaceID;
use replicore_store::ids::NamespacedResourceID;
use replicore_store::query::ClusterSpecEntryStream;

const DELETE_SQL: &str = r#"
DELETE FROM store_cluster_spec
WHERE
    ns_id = ?1
    AND cluster_id = ?2
;"#;

const LIST_SQL: &str = r#"
SELECT cluster_spec
FROM store_cluster_spec
WHERE ns_id = ?1
ORDER BY cluster_id ASC;
"#;

const LOOKUP_SQL: &str = r#"
SELECT cluster_spec
FROM store_cluster_spec
WHERE
    ns_id = ?1
    AND cluster_id = ?2
;"#;

const PERSIST_SQL: &str = r#"
INSERT INTO store_cluster_spec (ns_id, cluster_id, cluster_spec)
VALUES (?1, ?2, ?3)
ON CONFLICT(ns_id, cluster_id)
DO UPDATE SET
    cluster_spec=?3
;"#;

/// Delete a cluster specification from the store, ignoring missing clusters.
pub async fn delete(
    _: &Context,
    connection: &Connection,
    cluster: DeleteClusterSpec,
) -> Result<()> {
    let (err_count, _timer) = crate::telemetry::observe_op("clusterSpec.delete");
    let trace = crate::telemetry::trace_op("clusterSpec.delete");
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

/// Return a list of known [`ClusterSpec`] IDs in the given namespace.
pub async fn list(
    _: &Context,
    connection: &Connection,
    ns: NamespaceID,
) -> Result<ClusterSpecEntryStream> {
    let (err_count, _timer) = crate::telemetry::observe_op("clusterSpec.listIds");
    let trace = crate::telemetry::trace_op("clusterSpec.listIds");
    let cluster_specs = connection
        .call(move |connection| {
            let mut statement = connection.prepare_cached(LIST_SQL)?;
            let mut rows = statement.query([ns.id])?;

            let mut cluster_specs = Vec::new();
            while let Some(row) = rows.next()? {
                let cluster_spec: String = row.get("cluster_spec")?;
                cluster_specs.push(cluster_spec);
            }
            Ok(cluster_specs)
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await?;

    let cluster_specs = futures::stream::iter(cluster_specs)
        .map(|cluster_spec| {
            let cluster_spec = replisdk::utils::encoding::decode_serde(&cluster_spec)?;
            Ok(cluster_spec)
        })
        .boxed();
    Ok(cluster_specs)
}

/// Lookup a platform from the store, if one is available.
pub async fn lookup(
    _: &Context,
    connection: &Connection,
    cluster: NamespacedResourceID,
) -> Result<Option<ClusterSpec>> {
    let (err_count, timer) = crate::telemetry::observe_op("clusterSpec.lookup");
    let trace = crate::telemetry::trace_op("clusterSpec.lookup");
    let cluster = connection
        .call(move |connection| {
            let mut statement = connection.prepare_cached(LOOKUP_SQL)?;
            let mut rows = statement.query([cluster.ns_id, cluster.name])?;
            let row = match rows.next()? {
                None => None,
                Some(row) => {
                    let cluster: String = row.get("cluster_spec")?;
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

/// Persist a new or updated record into the store.
pub async fn persist(_: &Context, connection: &Connection, cluster: ClusterSpec) -> Result<()> {
    let record = replisdk::utils::encoding::encode_serde(&cluster)?;
    let (err_count, _timer) = crate::telemetry::observe_op("clusterSpec.persist");
    let trace = crate::telemetry::trace_op("clusterSpec.persist");
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

#[cfg(test)]
mod tests {
    use futures::TryStreamExt;

    use replicore_store::ids::NamespaceID;
    use replicore_store::ids::NamespacedResourceID;
    use replicore_store::query::LookupClusterSpec;

    use super::ClusterSpec;

    /// Return a [`ClusterSpec`] object to use in tests.
    fn mock_spec(name: &str) -> ClusterSpec {
        ClusterSpec::synthetic("test", name)
    }

    #[tokio::test]
    async fn delete_get_persist() {
        let context = replicore_context::Context::fixture();
        let store = crate::statements::tests::store().await;
        let cluster = mock_spec("ephemeral");
        let lookup = NamespacedResourceID {
            name: cluster.cluster_id.clone(),
            ns_id: cluster.ns_id.clone(),
        };
        let lookup = LookupClusterSpec(lookup);

        // Check lookup without record.
        let record = store
            .query(&context, lookup.clone())
            .await
            .expect("store lookup failed");
        assert_eq!(record.is_none(), true);

        // Check deleting without record.
        store.delete(&context, &cluster).await.unwrap();

        // Check persisting (and looking up) a record.
        store.persist(&context, cluster.clone()).await.unwrap();
        let record = store
            .query(&context, lookup.clone())
            .await
            .expect("store lookup failed")
            .expect("record not in store");
        assert_eq!(record.cluster_id, "ephemeral");
        assert_eq!(record.ns_id, "test");

        // Check deleting a record.
        store.delete(&context, &cluster).await.unwrap();
        let record = store
            .query(&context, lookup)
            .await
            .expect("store lookup failed");
        assert_eq!(record.is_none(), true);
    }

    #[tokio::test]
    async fn list() {
        let context = replicore_context::Context::fixture();
        let store = crate::statements::tests::store().await;

        // Fill the store with a few namespaces.
        store
            .persist(&context, mock_spec("cluster-1"))
            .await
            .unwrap();
        store
            .persist(&context, mock_spec("cluster-2"))
            .await
            .unwrap();
        store
            .persist(&context, mock_spec("cluster-3"))
            .await
            .unwrap();

        // Grab the list of IDs and check them.
        let ns = NamespaceID { id: "test".into() };
        let op = replicore_store::query::ListClusterSpecs(ns);
        let mut result = store.query(&context, op).await.unwrap();

        let mut ids = Vec::new();
        while let Some(item) = result.try_next().await.unwrap() {
            ids.push(item.cluster_id);
        }

        assert_eq!(ids, ["cluster-1", "cluster-2", "cluster-3"]);
    }
}
