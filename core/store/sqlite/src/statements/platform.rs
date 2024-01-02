//! Persistent store operations on Platforms.
use anyhow::Result;
use futures::StreamExt;
use opentelemetry_api::trace::FutureExt;
use tokio_rusqlite::Connection;

use replisdk::core::models::api::PlatformEntry;
use replisdk::core::models::platform::Platform;
use replisdk::utils::metrics::CountFutureErrExt;
use replisdk::utils::trace::TraceFutureStdErrExt;

use replicore_context::Context;
use replicore_store::delete::DeletePlatform;
use replicore_store::ids::NamespaceID;
use replicore_store::ids::NamespacedResourceID;
use replicore_store::query::PlatformEntryStream;

const DELETE_SQL: &str = r#"
DELETE FROM store_platform
WHERE
    ns_id = ?1
    AND name = ?2
;"#;

const LIST_SQL: &str = r#"
SELECT name, active
FROM store_platform
WHERE ns_id = ?1
ORDER BY name ASC;
"#;

const LOOKUP_SQL: &str = r#"
SELECT platform
FROM store_platform
WHERE
    ns_id = ?1
    AND name = ?2
;"#;

const PERSIST_SQL: &str = r#"
INSERT INTO store_platform (ns_id, name, platform)
VALUES (?1, ?2, ?3)
ON CONFLICT(ns_id, name)
DO UPDATE SET
    platform=?3
;"#;

/// Delete a platform from the store, ignoring missing platforms.
pub async fn delete(_: &Context, connection: &Connection, platform: DeletePlatform) -> Result<()> {
    let (err_count, _timer) = crate::telemetry::observe_op("platform.delete");
    let trace = crate::telemetry::trace_op("platform.delete");
    connection
        .call(move |connection| {
            connection.execute(
                DELETE_SQL,
                rusqlite::params![platform.0.ns_id, platform.0.name],
            )?;
            Ok(())
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await?;
    Ok(())
}

/// Return a list of known [`Platform`] IDs in the given namespace.
pub async fn list(
    _: &Context,
    connection: &Connection,
    ns: NamespaceID,
) -> Result<PlatformEntryStream> {
    let (err_count, _timer) = crate::telemetry::observe_op("platform.listIds");
    let trace = crate::telemetry::trace_op("platform.listIds");
    let items = connection
        .call(move |connection| {
            let mut statement = connection.prepare_cached(LIST_SQL)?;
            let mut rows = statement.query([ns.id])?;

            let mut items = Vec::new();
            while let Some(row) = rows.next()? {
                let active: bool = row.get("active")?;
                let name: String = row.get("name")?;
                let item = PlatformEntry { active, name };
                items.push(item);
            }
            Ok(items)
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await?;

    let items = futures::stream::iter(items).map(Ok).boxed();
    Ok(items)
}

/// Lookup a platform from the store, if one is available.
pub async fn lookup(
    _: &Context,
    connection: &Connection,
    pl: NamespacedResourceID,
) -> Result<Option<Platform>> {
    let (err_count, timer) = crate::telemetry::observe_op("platform.lookup");
    let trace = crate::telemetry::trace_op("platform.lookup");
    let platform = connection
        .call(move |connection| {
            let mut statement = connection.prepare_cached(LOOKUP_SQL)?;
            let mut rows = statement.query([pl.ns_id, pl.name])?;
            let row = match rows.next()? {
                None => None,
                Some(row) => {
                    let platform: String = row.get("platform")?;
                    Some(platform)
                }
            };
            Ok(row)
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await?;

    drop(timer);
    match platform {
        None => Ok(None),
        Some(platform) => {
            let platform = replisdk::utils::encoding::decode_serde(&platform)?;
            Ok(Some(platform))
        }
    }
}

/// Persist a new or updated record into the store.
pub async fn persist(_: &Context, connection: &Connection, platform: Platform) -> Result<()> {
    let record = replisdk::utils::encoding::encode_serde(&platform)?;
    let (err_count, _timer) = crate::telemetry::observe_op("platform.persist");
    let trace = crate::telemetry::trace_op("platform.persist");
    connection
        .call(move |connection| {
            connection.execute(
                PERSIST_SQL,
                rusqlite::params![platform.ns_id, platform.name, record],
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

    use replisdk::core::models::platform::PlatformTransport;
    use replisdk::core::models::platform::PlatformTransportHttp;

    use replicore_store::ids::NamespaceID;
    use replicore_store::ids::NamespacedResourceID;
    use replicore_store::query::LookupPlatform;

    use super::Platform;

    /// Return a [`Platform`] object to use in tests.
    fn mock_platform(name: &str) -> Platform {
        Platform {
            name: name.into(),
            ns_id: "test".into(),
            active: true,
            discovery: Default::default(),
            transport: PlatformTransport::Http(PlatformTransportHttp {
                base_url: format!("https://{}.test", name),
                tls_ca_bundle: None,
                tls_insecure_skip_verify: false,
            }),
        }
    }

    #[tokio::test]
    async fn delete_get_persist() {
        let context = replicore_context::Context::fixture();
        let store = crate::statements::tests::store().await;
        let platform = mock_platform("ephemeral");
        let lookup = NamespacedResourceID {
            name: platform.name.clone(),
            ns_id: platform.ns_id.clone(),
        };
        let lookup = LookupPlatform(lookup);

        // Check lookup without record.
        let record = store
            .query(&context, lookup.clone())
            .await
            .expect("store lookup failed");
        assert_eq!(record.is_none(), true);

        // Check deleting without record.
        store.delete(&context, &platform).await.unwrap();

        // Check persisting (and looking up) a record.
        store.persist(&context, platform.clone()).await.unwrap();
        let record = store
            .query(&context, lookup.clone())
            .await
            .expect("store lookup failed")
            .expect("ns record not in store");
        assert_eq!(record.name, "ephemeral");
        assert_eq!(record.ns_id, "test");

        // Check deleting a record.
        store.delete(&context, &platform).await.unwrap();
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
            .persist(&context, mock_platform("node-1"))
            .await
            .unwrap();
        store
            .persist(&context, mock_platform("node-2"))
            .await
            .unwrap();
        store
            .persist(&context, mock_platform("node-3"))
            .await
            .unwrap();

        // Grab the list of IDs and check them.
        let ns = NamespaceID { id: "test".into() };
        let op = replicore_store::query::ListPlatforms(ns);
        let mut result = store.query(&context, op).await.unwrap();

        let mut ids = Vec::new();
        while let Some(item) = result.try_next().await.unwrap() {
            ids.push(item.name);
        }

        assert_eq!(ids, ["node-1", "node-2", "node-3"]);
    }
}
