//! Persistent store operations on Orchestrate Reports.
use anyhow::Result;
use opentelemetry_api::trace::FutureExt;
use tokio_rusqlite::Connection;

use replisdk::utils::metrics::CountFutureErrExt;
use replisdk::utils::trace::TraceFutureStdErrExt;

use replicore_cluster_models::OrchestrateReport;
use replicore_context::Context;
use replicore_store::ids::NamespacedResourceID;

const LOOKUP_SQL: &str = r#"
SELECT report
FROM store_orchestrate_report
WHERE
    ns_id = ?1
    AND cluster_id = ?2
;"#;

const PERSIST_SQL: &str = r#"
INSERT INTO store_orchestrate_report (ns_id, cluster_id, report)
VALUES (?1, ?2, ?3)
ON CONFLICT(ns_id, cluster_id)
DO UPDATE SET
    report=?3
;"#;

/// Lookup an orchestate report from the store, if one is available.
pub async fn lookup(
    _: &Context,
    connection: &Connection,
    cluster: NamespacedResourceID,
) -> Result<Option<OrchestrateReport>> {
    let (err_count, timer) = crate::telemetry::observe_op("orchestrateReport.lookup");
    let trace = crate::telemetry::trace_op("orchestrateReport.lookup");
    let report = connection
        .call(move |connection| {
            let mut statement = connection.prepare_cached(LOOKUP_SQL)?;
            let mut rows = statement.query([cluster.ns_id, cluster.name])?;
            let row = match rows.next()? {
                None => None,
                Some(row) => {
                    let report: String = row.get("report")?;
                    Some(report)
                }
            };
            Ok(row)
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await?;

    drop(timer);
    match report {
        None => Ok(None),
        Some(report) => {
            let report = replisdk::utils::encoding::decode_serde(&report)?;
            Ok(Some(report))
        }
    }
}

/// Persist a new or updated [`OrchestrateReport`]` into the store.
pub async fn persist(
    _: &Context,
    connection: &Connection,
    report: OrchestrateReport,
) -> Result<()> {
    let record = replisdk::utils::encoding::encode_serde(&report)?;
    let (err_count, _timer) = crate::telemetry::observe_op("orchestrateReport.persist");
    let trace = crate::telemetry::trace_op("orchestrateReport.persist");
    connection
        .call(move |connection| {
            connection.execute(
                PERSIST_SQL,
                rusqlite::params![report.ns_id, report.cluster_id, record],
            )?;
            Ok(())
        })
        .count_on_err(err_count)
        .trace_on_err_with_status()
        .with_context(trace)
        .await?;
    Ok(())
}
