use failure::ResultExt;
use opentracingrust::Span;

use replicante_models_core::cluster::OrchestrateReport;
use replicante_models_core::events::cluster::ClusterEvent;

use crate::follower::Follower;
use crate::ErrorKind;
use crate::Result;

/// Extract and persist action information.
pub fn process(follower: &Follower, event: &ClusterEvent, span: Option<&mut Span>) -> Result<()> {
    match event {
        ClusterEvent::OrchestrateReport(report) => persist_report(follower, report, span),
        _ => Ok(()),
    }
}

/// Helper function to persist an orchestrate report to the view store.
fn persist_report(
    follower: &Follower,
    report: &OrchestrateReport,
    span: Option<&mut Span>,
) -> Result<()> {
    follower
        .store
        .persist()
        .cluster_orchestrate_report(
            report.clone(),
            span.as_ref().map(|span| span.context().clone()),
        )
        .with_context(|_| ErrorKind::StoreWrite("orchestrate report"))?;
    Ok(())
}
