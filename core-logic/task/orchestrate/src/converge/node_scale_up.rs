//! Check the number of nodes in the cluster and create new ones if needed.
use std::collections::HashMap;
use std::time::Duration;

use anyhow::Result;

use replicore_cluster_models::ConvergeState;
use replicore_context::Context;

use super::step::ConvergeStep;
use super::ConvergeData;

static SCALE_UP_GRACE_ID: &str = "node-scale-up";

/// Provision new cluster nodes if the declared shape does not match the discovered one.
pub struct NodeScaleUp;

#[async_trait::async_trait]
impl ConvergeStep for NodeScaleUp {
    async fn converge(
        &self,
        context: &Context,
        data: &ConvergeData,
        state: &mut ConvergeState,
    ) -> Result<()> {
        slog::trace!(
            context.logger, "Checking node scale up for cluster";
            "ns_id" => &data.ns.id,
            "cluster_id" => &data.cluster_current.spec.cluster_id,
        );

        // Skip step if cluster has no convergence configured.
        let declaration = match &data.cluster_current.spec.declaration {
            None => return Ok(()),
            Some(declaration) => declaration,
        };

        // Skip step if last scale up triggered too recently.
        if let Some(grace) = state.graces.get(SCALE_UP_GRACE_ID) {
            // TODO: scale up grace should be configurable.
            let grace_lap = Duration::from_secs(5 * 60);
            if *grace + grace_lap > time::OffsetDateTime::now_utc() {
                return Ok(());
            }
        }
        state.graces.remove(SCALE_UP_GRACE_ID);

        // Skip if cluster discovery matches.
        let mut counts: HashMap<&String, u32> = HashMap::new();
        for node in &data.cluster_current.discovery.nodes {
            if let Some(group) = &node.node_group {
                let count = counts.entry(group).or_insert(0);
                *count += 1;
            }
        }
        let mut skip = true;
        for (group_id, group) in &declaration.nodes {
            let actual = counts.get(group_id).copied().unwrap_or(0);
            let expected = group.desired_count;
            skip = skip && (actual == expected);
        }
        if skip {
            return Ok(());
        }

        // TODO: Skip if running (new node? all?) provisioning action.
        // TODO: Schedule action to create a new node.
        // TODO: Update convergence record to suspect scale up activity as configured.
        Ok(())
    }
}
