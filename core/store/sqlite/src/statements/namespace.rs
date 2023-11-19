//! Persistent store operations on Namespaces.
use anyhow::Result;
use opentelemetry_api::trace::FutureExt;
use tokio_rusqlite::Connection;

use replisdk::core::models::namespace::Namespace;
use replisdk::utils::metrics::CountFutureErrExt;
use replisdk::utils::trace::TraceFutureStdErrExt;

use replicore_context::Context;
use replicore_store::delete::DeleteNamespace;
use replicore_store::query::LookupNamespace;

const DELETE_SQL: &str = r#"
DELETE FROM store_namespace
WHERE id = ?1;
"#;

const LOOKUP_SQL: &str = r#"
SELECT namespace
FROM store_namespace
WHERE id = ?1;
"#;

const PERSIST_SQL: &str = r#"
INSERT INTO store_namespace (namespace, id)
VALUES (?1, ?2);
ON CONFLICT(id)
DO UPDATE SET
    namespace=?1,
;
"#;

/// Delete a namespace from the store, ignoring missing namespaces.
pub async fn delete(_: &Context, connection: &Connection, ns: DeleteNamespace) -> Result<()> {
    let (err_count, _timer) = crate::telemetry::observe_op("namespace.delete");
    let trace = crate::telemetry::trace_op("namespace.delete");
    connection
        .call(move |connection| {
            connection.execute(DELETE_SQL, rusqlite::params![ns.id])?;
            Ok(())
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await?;
    Ok(())
}

// TODO: list

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
            let mut rows = statement.query([ns.id])?;
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
            connection.execute(PERSIST_SQL, rusqlite::params![record, &ns.id])?;
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
    use super::LookupNamespace;
    use super::Namespace;

    #[tokio::test]
    async fn operations() {
        let context = replicore_context::Context::fixture();
        let store = crate::statements::tests::store().await;
        let ns = Namespace { id: "test".into() };
        let lookup = LookupNamespace { id: "test".into() };

        // Check lookup without record.
        let record = store
            .query(&context, lookup.clone())
            .await
            .expect("store lookup to pass");
        assert_eq!(record.is_none(), true);

        // Check deleting without record.
        store.delete(&context, &ns).await.unwrap();

        // Check persisting (and looking up) a record.
        store.persist(&context, ns.clone()).await.unwrap();
        let record = store
            .query(&context, lookup.clone())
            .await
            .expect("store lookup to pass")
            .expect("ns record not in store");
        assert_eq!(record.id, "test");

        // Check deleting a record.
        store.delete(&context, &ns).await.unwrap();
        let record = store
            .query(&context, lookup.clone())
            .await
            .expect("store lookup to pass");
        assert_eq!(record.is_none(), true);
    }

    // TODO: test listing
}
