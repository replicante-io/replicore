//! Check the number of nodes in the cluster and create new ones if needed.
use std::collections::HashMap;
use std::time::Duration;

use anyhow::Result;

use replisdk::core::models::api::OActionSpec;
use replisdk::platform::models::NodeProvisionRequestDetails;

use replicore_cluster_models::ConvergeState;
use replicore_context::Context;

use super::constants::ACTION_KIND_PROVISION;
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
        let declaration = &data.cluster_current.spec.declaration;
        if !declaration.active {
            slog::debug!(
                context.logger, "Skip node scale up for inactive cluster";
                "ns_id" => &data.ns.id,
                "cluster_id" => &data.cluster_current.spec.cluster_id,
            );
            return Ok(());
        }
        let definition = match &declaration.definition {
            Some(definition) => definition,
            None => {
                slog::debug!(
                    context.logger, "Skip node scale up for undeclared cluster";
                    "ns_id" => &data.ns.id,
                    "cluster_id" => &data.cluster_current.spec.cluster_id,
                );
                return Ok(());
            }
        };

        // Skip step if last scale up triggered too recently.
        if let Some(grace) = state.graces.get(SCALE_UP_GRACE_ID) {
            let grace_time = declaration.grace_up;
            let grace_time = Duration::from_secs(grace_time * 60);
            if *grace + grace_time > time::OffsetDateTime::now_utc() {
                slog::debug!(
                    context.logger, "Skip node scale up while in grace period";
                    "ns_id" => &data.ns.id,
                    "cluster_id" => &data.cluster_current.spec.cluster_id,
                );
                return Ok(());
            }
        }
        state.graces.remove(SCALE_UP_GRACE_ID);

        // Skip step in case of unfinished provisioning actions.
        let scaling = data
            .cluster_current
            .oactions_unfinished
            .iter()
            .any(|oaction| oaction.kind == ACTION_KIND_PROVISION);
        if scaling {
            slog::debug!(
                context.logger, "Skip node scale up due to scaling activity";
                "ns_id" => &data.ns.id,
                "cluster_id" => &data.cluster_current.spec.cluster_id,
            );
            return Ok(());
        }

        // Skip if cluster discovery matches.
        let mut counts: HashMap<&String, u32> = HashMap::new();
        for node in &data.cluster_current.discovery.nodes {
            if let Some(group) = &node.node_group {
                let count = counts.entry(group).or_insert(0);
                *count += 1;
            }
        }
        let partial_groups: Vec<_> = definition
            .nodes
            .iter()
            .filter(|(group_id, group)| {
                let actual = counts.get(group_id).copied().unwrap_or(0);
                actual != group.desired_count
            })
            .collect();
        if partial_groups.is_empty() {
            return Ok(());
        }

        // Schedule action to create a new nodes.
        let (node_group_id, _) = partial_groups[0];
        let args = NodeProvisionRequestDetails {
            // Initially only provision one node at a time, we'll figure out bulk in the future.
            count: 1u16,
            node_group_id: node_group_id.to_string(),
        };
        let node_up = OActionSpec {
            ns_id: data.cluster_current.spec.ns_id.clone(),
            cluster_id: data.cluster_current.spec.cluster_id.clone(),
            action_id: None,
            args: serde_json::to_value(args)?,
            approval: declaration.approval,
            kind: String::from(ACTION_KIND_PROVISION),
            metadata: Default::default(),
            timeout: None,
        };
        let sdk = replicore_sdk::CoreSDK::from_injector(data.injector.clone());
        sdk.oaction_create(context, node_up).await?;

        // Update convergence state to make information available to the next loop.
        state.graces.insert(SCALE_UP_GRACE_ID.to_string(), time::OffsetDateTime::now_utc());
        // TODO? track the action to wait it out.
        Ok(())
    }
}
