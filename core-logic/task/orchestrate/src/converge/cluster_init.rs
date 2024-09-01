//! Initialise a cluster with no initialised nodes, if needed.
use std::time::Duration;

use anyhow::Result;

use replisdk::core::models::api::NActionSpec;
use replisdk::core::models::cluster::ClusterSpecInitMode;
use replisdk::core::models::node::NodeStatus;

use replicore_cluster_models::ConvergeState;
use replicore_cluster_models::OrchestrateReportNote;
use replicore_context::Context;

use super::constants::ACTION_KIND_CLUSTER_INIT;
use super::constants::STEP_ID_CLUSTER_INIT;
use super::step::ConvergeStep;
use super::ConvergeData;

/// Initialise a cluster with no initialised nodes, if needed.
pub struct ClusterInit;

#[async_trait::async_trait]
impl ConvergeStep for ClusterInit {
    async fn converge(
        &self,
        context: &Context,
        data: &ConvergeData,
        state: &mut ConvergeState,
    ) -> Result<()> {
        slog::trace!(
            context.logger, "Checking cluster for first time initialisation";
            "ns_id" => data.ns_id(),
            "cluster_id" => data.cluster_id(),
        );

        // Skip initialisation if the cluster init mode is NotManaged.
        let declaration = &data.cluster_new.spec.declaration;
        let initialise = &declaration.initialise;
        if matches!(initialise.mode, ClusterSpecInitMode::NotManaged) {
            slog::debug!(
                context.logger, "Skip cluster initialisation due to not-managed mode";
                "ns_id" => data.ns_id(),
                "cluster_id" => data.cluster_id(),
            );
            return Ok(());
        }

        // Skip initialisation it the last attempt was too recent.
        if let Some(grace) = state.graces.get(STEP_ID_CLUSTER_INIT) {
            let grace_time = initialise.grace;
            let grace_time = Duration::from_secs(grace_time * 60);
            let grace_expire = *grace + grace_time;
            if grace_expire > time::OffsetDateTime::now_utc() {
                slog::debug!(
                    context.logger, "Skip cluster initialisation request while in grace period";
                    "ns_id" => data.ns_id(),
                    "cluster_id" => data.cluster_id(),
                );
                return Ok(());
            }
        }
        state.graces.remove(STEP_ID_CLUSTER_INIT);

        // Skip initialisation if cluster has no nodes.
        if data.cluster_new.discovery.nodes.is_empty() {
            slog::debug!(
                context.logger, "Skip cluster initialisation for cluster with no nodes";
                "ns_id" => data.ns_id(),
                "cluster_id" => data.cluster_id(),
            );
            return Ok(());
        }

        // Only initialise if all cluster nodes are out of cluster.
        let nodes_all_out_of_cluster = data
            .cluster_new
            .nodes
            .values()
            .all(|node| matches!(node.node_status, NodeStatus::NotInCluster));
        if !nodes_all_out_of_cluster {
            slog::debug!(
                context.logger,
                "Skip cluster initialisation for clusters with nodes not out of cluster";
                "ns_id" => data.ns_id(),
                "cluster_id" => data.cluster_id(),
            );
            return Ok(());
        }

        // Skip initialisation if an init action exists for any node.
        let any_init_action = data
            .cluster_new
            .index_nactions_by_id
            .values()
            .any(|action| action.kind == ACTION_KIND_CLUSTER_INIT);
        if any_init_action {
            slog::debug!(
                context.logger,
                "Skip cluster initialisation as an init action is still unfinished";
                "ns_id" => data.ns_id(),
                "cluster_id" => data.cluster_id(),
            );
            return Ok(());
        }

        // Pick a node to target.
        let target = match &initialise.search {
            None => data.cluster_new.search_nodes(&Default::default())?,
            Some(search) => data.cluster_new.search_nodes(search)?,
        };
        let target = target.one().ok_or(super::errors::ClusterInitNoTarget)?;

        // Schedule the cluster.init action.
        let spec = NActionSpec {
            ns_id: target.ns_id.clone(),
            cluster_id: target.cluster_id.clone(),
            node_id: target.node_id.clone(),
            action_id: None,
            args: initialise.action_args.clone(),
            approval: declaration.approval,
            kind: ACTION_KIND_CLUSTER_INIT.to_string(),
            metadata: Default::default(),
        };
        let sdk = replicore_sdk::CoreSDK::from(&data.injector);
        let action = sdk.naction_create(context, spec).await?;
        let mut note = OrchestrateReportNote::decision("Scheduled cluster initialisation on node");
        note.for_node(&action.node_id)
            .for_node_action(action.action_id);
        data.report_mut().notes.push(note);
        slog::debug!(
            context.logger, "Scheduled cluster initialisation on node";
            "ns_id" => data.ns_id(),
            "cluster_id" => data.cluster_id(),
            "node_id" => &action.node_id,
            "action_id" => %action.action_id,
        );

        // Update convergence state to make information available to the next loop.
        state.graces.insert(
            STEP_ID_CLUSTER_INIT.to_string(),
            time::OffsetDateTime::now_utc(),
        );
        Ok(())
    }
}
