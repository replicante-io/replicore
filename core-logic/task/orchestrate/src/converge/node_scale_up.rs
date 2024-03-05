//! Check the number of nodes in the cluster and create new ones if needed.
use anyhow::Result;

use replicore_context::Context;

use super::step::ConvergeStep;
use crate::init::InitData;

/// Provision new cluster nodes if the declared shape does not match the discovered one.
pub struct NodeScaleUp;

#[async_trait::async_trait]
impl ConvergeStep for NodeScaleUp {
    async fn converge(&self, context: &Context, data: &InitData) -> Result<()> {
        slog::debug!(
            context.logger, "Checking node scale up for cluster";
            "ns_id" => &data.ns.id,
            "cluster_id" => &data.cluster_current.spec.cluster_id,
        );
        // TODO: Skip if cluster has no convergence configured.
        // TODO: Skip if cluster discovery matches.
        // TODO: Skip if running (new node? all?) provisioning action.
        // TODO: Skip if last scale up triggered too recently.
        // TODO: Schedule action to create a new node.
        // TODO: Update convergence record to suspect scale up activity as configured.
        Ok(())
    }
}
