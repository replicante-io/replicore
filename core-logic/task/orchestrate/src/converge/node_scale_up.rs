//! Check the number of nodes in the cluster and create new ones if needed.
use std::collections::HashMap;
use std::time::Duration;

use anyhow::Result;

use replisdk::core::models::api::OActionSpec;
use replisdk::platform::models::NodeProvisionRequestDetails;

use replicore_cluster_models::ConvergeState;
use replicore_cluster_models::OrchestrateReportNote;
use replicore_context::Context;
use replicore_oaction_platform::ProvisionNodesArgs;

use super::constants::ACTION_KIND_PROVISION;
use super::constants::STEP_ID_SCALE_UP;
use super::step::ConvergeStep;
use super::ConvergeData;

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
            "ns_id" => data.ns_id(),
            "cluster_id" => data.cluster_id(),
        );

        // Skip step if cluster has no convergence configured.
        let declaration = &data.cluster_new.spec.declaration;
        let definition = match &declaration.definition {
            Some(definition) => definition,
            None => {
                slog::debug!(
                    context.logger, "Skip node scale up without cluster definition";
                    "ns_id" => data.ns_id(),
                    "cluster_id" => data.cluster_id(),
                );
                return Ok(());
            }
        };

        // Skip step if last scale up triggered too recently.
        if let Some(grace) = state.graces.get(STEP_ID_SCALE_UP) {
            let grace_time = declaration.graces.scale_up;
            let grace_time = Duration::from_secs(grace_time * 60);
            if *grace + grace_time > time::OffsetDateTime::now_utc() {
                slog::debug!(
                    context.logger, "Skip node scale up while in grace period";
                    "ns_id" => data.ns_id(),
                    "cluster_id" => data.cluster_id(),
                );
                return Ok(());
            }
        }
        state.graces.remove(STEP_ID_SCALE_UP);

        // Skip step in case of unfinished provisioning actions.
        let scaling = data
            .cluster_new
            .oactions_unfinished
            .iter()
            .any(|oaction| oaction.kind == ACTION_KIND_PROVISION);
        if scaling {
            slog::debug!(
                context.logger, "Skip node scale up due to scaling activity";
                "ns_id" => data.ns_id(),
                "cluster_id" => data.cluster_id(),
            );
            return Ok(());
        }

        // Skip if cluster has enough nodes.
        let mut counts: HashMap<&String, u32> = HashMap::new();
        for node in &data.cluster_new.discovery.nodes {
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
                actual < group.desired_count
            })
            .collect();
        if partial_groups.is_empty() {
            slog::debug!(
                context.logger, "Skip node scale up since all groups meet the desired count";
                "ns_id" => data.ns_id(),
                "cluster_id" => data.cluster_id(),
            );
            return Ok(());
        }

        // Schedule action to create a new nodes.
        let (node_group_id, _) = partial_groups[0];
        // Initially fix provisioning options, we'll figure out bulk create and more in the future.
        let args = NodeProvisionRequestDetails {
            count: 1u16,
            node_group_id: node_group_id.to_string(),
        };
        let args = ProvisionNodesArgs::from(args);
        let node_up = OActionSpec {
            ns_id: data.ns_id().to_string(),
            cluster_id: data.cluster_id().to_string(),
            action_id: None,
            args: serde_json::to_value(args)?,
            approval: declaration.approval,
            kind: String::from(ACTION_KIND_PROVISION),
            metadata: Default::default(),
            timeout: None,
        };
        let sdk = replicore_sdk::CoreSDK::from(&data.injector);
        let action = sdk.oaction_create(context, node_up).await?;
        let note = OrchestrateReportNote::decision("Cluster scale-up action scheduled");
        data.report_mut().notes.push(note);
        slog::debug!(
            context.logger, "Cluster scale-up action scheduled";
            "ns_id" => data.ns_id(),
            "cluster_id" => data.cluster_id(),
            "action_id" => %action.action_id,
        );

        // Update convergence state to make information available to the next loop.
        state.graces.insert(
            STEP_ID_SCALE_UP.to_string(),
            time::OffsetDateTime::now_utc(),
        );
        Ok(())
    }
}
