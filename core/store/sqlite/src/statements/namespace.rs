//! Persistent store operations on Namespaces.
use anyhow::Result;
use futures::StreamExt;
use opentelemetry_api::trace::FutureExt;
use tokio_rusqlite::Connection;

use replisdk::core::models::namespace::Namespace;
use replisdk::utils::metrics::CountFutureErrExt;
use replisdk::utils::trace::TraceFutureStdErrExt;

use replicore_context::Context;
use replicore_store::delete::DeleteNamespace;
use replicore_store::query::LookupNamespace;
use replicore_store::query::StringStream;

const DELETE_SQL: &str = r#"
DELETE FROM store_namespace
WHERE id = ?1;
"#;

const LIST_IDS_SQL: &str = r#"
SELECT id
FROM store_namespace
ORDER BY id ASC;
"#;

const LOOKUP_SQL: &str = r#"
SELECT namespace
FROM store_namespace
WHERE id = ?1;
"#;

const PERSIST_SQL: &str = r#"
INSERT INTO store_namespace (id, namespace)
VALUES (?1, ?2);
ON CONFLICT(id)
DO UPDATE SET
    namespace=?2,
;"#;

/// Delete a namespace from the store, ignoring missing namespaces.
pub async fn delete(_: &Context, connection: &Connection, ns: DeleteNamespace) -> Result<()> {
    let (err_count, _timer) = crate::telemetry::observe_op("namespace.delete");
    let trace = crate::telemetry::trace_op("namespace.delete");
    connection
        .call(move |connection| {
            connection.execute(DELETE_SQL, rusqlite::params![ns.0.id])?;
            Ok(())
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await?;
    Ok(())
}

/// Return a list of known [`Namespace`] IDs.
pub async fn list(_: &Context, connection: &Connection) -> Result<StringStream> {
    let (err_count, _timer) = crate::telemetry::observe_op("namespace.listIds");
    let trace = crate::telemetry::trace_op("namespace.listIds");
    let ids = connection
        .call(move |connection| {
            let mut statement = connection.prepare_cached(LIST_IDS_SQL)?;
            let mut rows = statement.query([])?;

            let mut ids = Vec::new();
            while let Some(row) = rows.next()? {
                let id: String = row.get("id")?;
                ids.push(id);
            }
            Ok(ids)
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await?;

    let ids = futures::stream::iter(ids).map(Ok).boxed();
    Ok(ids)
}

/// Lookup a namespace from the store, if one is available.
pub async fn lookup(
    _: &Context,
    connection: &Connection,
    ns: LookupNamespace,
) -> Result<Option<Namespace>> {
    let (err_count, timer) = crate::telemetry::observe_op("namespace.lookup");
    let trace = crate::telemetry::trace_op("namespace.lookup");
    let namespace = connection
        .call(move |connection| {
            let mut statement = connection.prepare_cached(LOOKUP_SQL)?;
            let mut rows = statement.query([ns.0.id])?;
            let row = match rows.next()? {
                None => None,
                Some(row) => {
                    let namespace: String = row.get("namespace")?;
                    Some(namespace)
                }
            };
            Ok(row)
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await?;

    drop(timer);
    match namespace {
        None => Ok(None),
        Some(namespace) => {
            let ns = replisdk::utils::encoding::decode_serde(&namespace)?;
            Ok(Some(ns))
        }
    }
}

/// Persist a new or updated record into the store.
pub async fn persist(_: &Context, connection: &Connection, ns: Namespace) -> Result<()> {
    let record = replisdk::utils::encoding::encode_serde(&ns)?;
    let (err_count, _timer) = crate::telemetry::observe_op("namespace.persist");
    let trace = crate::telemetry::trace_op("namespace.persist");
    connection
        .call(move |connection| {
            connection.execute(PERSIST_SQL, rusqlite::params![ns.id, record])?;
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

    use replisdk::core::models::namespace::NamespaceStatus;

    use replicore_store::ids::NamespaceID;

    use super::LookupNamespace;
    use super::Namespace;

    fn mock_namespace(id: &str) -> Namespace {
        Namespace {
            id: id.into(),
            tls: Default::default(),
            status: NamespaceStatus::Active,
        }
    }

    #[tokio::test]
    async fn delete_get_persist() {
        let context = replicore_context::Context::fixture();
        let store = crate::statements::tests::store().await;
        let ns = mock_namespace("test");
        let lookup = NamespaceID { id: "test".into() };
        let lookup = LookupNamespace(lookup);

        // Check lookup without record.
        let record = store
            .query(&context, lookup.clone())
            .await
            .expect("store lookup failed");
        assert_eq!(record.is_none(), true);

        // Check deleting without record.
        store.delete(&context, &ns).await.unwrap();

        // Check persisting (and looking up) a record.
        store.persist(&context, ns.clone()).await.unwrap();
        let record = store
            .query(&context, lookup.clone())
            .await
            .expect("store lookup failed")
            .expect("ns record not in store");
        assert_eq!(record.id, "test");

        // Check deleting a record.
        store.delete(&context, &ns).await.unwrap();
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
            .persist(&context, mock_namespace("test-1"))
            .await
            .unwrap();
        store
            .persist(&context, mock_namespace("test-2"))
            .await
            .unwrap();
        store
            .persist(&context, mock_namespace("test-3"))
            .await
            .unwrap();

        // Grab the list of IDs and check them.
        let op = replicore_store::query::ListNamespaceIds;
        let mut result = store.query(&context, op).await.unwrap();

        let mut ids = Vec::new();
        while let Some(id) = result.try_next().await.unwrap() {
            ids.push(id);
        }

        assert_eq!(ids, ["test-1", "test-2", "test-3"]);
    }
}
