//! Expand an initialised cluster with any NotInCluster nodes.
use anyhow::Result;

use replisdk::core::models::action::ActionApproval;
use replisdk::core::models::api::NActionSpec;
use replisdk::core::models::cluster::ClusterDeclaration;
use replisdk::core::models::cluster::ClusterDeclarationExpandMode as ExpandMode;
use replisdk::core::models::node::AttributeMatcher;
use replisdk::core::models::node::AttributeMatcherComplex;
use replisdk::core::models::node::Node;
use replisdk::core::models::node::NodeSearch;
use replisdk::core::models::node::NodeStatus;

use replicore_cluster_models::ConvergeState;
use replicore_cluster_models::OrchestrateReportNote;
use replicore_context::Context;

use super::constants::ACTION_KIND_CLUSTER_ADD;
use super::constants::ACTION_KIND_CLUSTER_JOIN;
use super::constants::STEP_ID_CLUSTER_EXPAND;
use super::step::ConvergeStep;
use super::ConvergeData;

/// Expand an initialised cluster with any NotInCluster nodes.
pub struct ClusterExpand;

#[async_trait::async_trait]
impl ConvergeStep for ClusterExpand {
    async fn converge(
        &self,
        context: &Context,
        data: &ConvergeData,
        state: &mut ConvergeState,
    ) -> Result<()> {
        slog::trace!(
            context.logger, "Checking cluster for node expansion";
            "ns_id" => data.ns_id(),
            "cluster_id" => data.cluster_id(),
        );

        // Skip if expand mode is Auto.
        let declaration = &data.cluster_new.spec.declaration;
        let expand = &declaration.expand;
        if matches!(expand.mode, ExpandMode::Auto) {
            slog::debug!(
                context.logger, "Skip cluster expand due to auto mode";
                "ns_id" => data.ns_id(),
                "cluster_id" => data.cluster_id(),
            );
            return Ok(());
        }

        // Skip if in expand grace period.
        let expand_grace = declaration.graces.expand;
        if super::step::grace_check(STEP_ID_CLUSTER_EXPAND, &state.graces, expand_grace) {
            slog::debug!(
                context.logger, "Skip cluster expand while in grace period";
                "ns_id" => data.ns_id(),
                "cluster_id" => data.cluster_id(),
            );
            return Ok(());
        }
        state.graces.remove(STEP_ID_CLUSTER_EXPAND);

        // Skip if no nodes is NotInCluster.
        let mut new_node = data
            .cluster_new
            .nodes
            .values()
            .filter(|node| matches!(node.node_status, NodeStatus::NotInCluster));
        let new_node = match new_node.next() {
            Some(node) => node,
            None => {
                slog::debug!(
                    context.logger, "Skip cluster expand when all nodes in cluster";
                    "ns_id" => data.ns_id(),
                    "cluster_id" => data.cluster_id(),
                );
                return Ok(());
            }
        };

        // Skip if cluster is being expanded already.
        let any_expand_action = data
            .cluster_new
            .index_nactions_by_id
            .values()
            .any(|action| {
                action.kind == ACTION_KIND_CLUSTER_ADD || action.kind == ACTION_KIND_CLUSTER_JOIN
            });
        if any_expand_action {
            slog::debug!(
                context.logger,
                "Skip cluster expand for cluster with an expand action is still unfinished";
                "ns_id" => data.ns_id(),
                "cluster_id" => data.cluster_id(),
            );
            return Ok(());
        }

        // Skip if a target member to expand from can't be found.
        let target = match &expand.target_member {
            Some(search) => data.cluster_new.search_nodes(search)?,
            None => {
                let mut search = NodeSearch::default();
                search
                    .matches
                    .insert("node_status".into(), AttributeMatcher::Eq("HEALTHY".into()));
                search.matches.insert(
                    "address.member".into(),
                    AttributeMatcher::Complex(AttributeMatcherComplex {
                        op: replisdk::core::models::node::AttributeMatcherOp::Set,
                        value: None,
                        values: None,
                    }),
                );
                search.matches.insert(
                    "shard.count.primary".into(),
                    AttributeMatcher::Complex(AttributeMatcherComplex {
                        op: replisdk::core::models::node::AttributeMatcherOp::Ne,
                        value: Some(replisdk::agent::models::AttributeValue::Number(
                            serde_json::Number::from(0),
                        )),
                        values: None,
                    }),
                );
                search.sort_by = vec![
                    String::from("-shard.count.primary"),
                    String::from("node_id"),
                ];
                data.cluster_new.search_nodes(&search)?
            }
        };
        let target = match target.one() {
            Some(target) => target,
            None => {
                slog::debug!(
                    context.logger, "Skip cluster expand since target member was not found";
                    "ns_id" => data.ns_id(),
                    "cluster_id" => data.cluster_id(),
                );
                let note = OrchestrateReportNote::decision(
                    "Not expanding cluster: could not find a healthy member to extend from",
                );
                data.report_mut().notes.push(note);
                return Ok(());
            }
        };

        // Schedule cluster expand action based on mode.
        let spec = expand_naction(declaration, new_node, &target)?;
        let sdk = replicore_sdk::CoreSDK::from(&data.injector);
        let action = sdk.naction_create(context, spec).await?;
        let mut note = OrchestrateReportNote::decision("Scheduled cluster expand action on node");
        note.for_node(&action.node_id)
            .for_node_action(action.action_id);
        data.report_mut().notes.push(note);
        slog::debug!(
            context.logger, "Scheduled cluster expand on node";
            "ns_id" => data.ns_id(),
            "cluster_id" => data.cluster_id(),
            "node_id" => &action.node_id,
            "action_id" => %action.action_id,
        );

        // Update convergence state.
        super::step::grace_start(STEP_ID_CLUSTER_EXPAND, &mut state.graces);
        Ok(())
    }
}

/// Generate the [`NActionSpec`] record to expand the cluster.
fn expand_naction(
    declaration: &ClusterDeclaration,
    new_node: &Node,
    target: &Node,
) -> Result<NActionSpec> {
    match declaration.expand.mode {
        ExpandMode::Add => expand_naction_add(declaration, new_node, target),
        ExpandMode::Auto => panic!("requested NActionSpec for auto expand cluster"),
        ExpandMode::Join => expand_naction_join(declaration, new_node, target),
    }
}

/// Generate the [`NActionSpec`] record to expand the cluster by addition (from an existing node).
fn expand_naction_add(
    declaration: &ClusterDeclaration,
    new_node: &Node,
    target: &Node,
) -> Result<NActionSpec> {
    let address = &new_node
        .details
        .as_ref()
        .ok_or_else(|| super::errors::NodeNoMemberAddress::from(new_node))?
        .address
        .member;
    let args = serde_json::json!({
        "node": address,
    });

    let kind = ACTION_KIND_CLUSTER_ADD.to_string();
    let spec = naction_spec(declaration.approval, args, target, kind);
    Ok(spec)
}

/// Generate the [`NActionSpec`] record to expand the cluster by joining (from the new node).
fn expand_naction_join(
    declaration: &ClusterDeclaration,
    new_node: &Node,
    target: &Node,
) -> Result<NActionSpec> {
    let address = &target
        .details
        .as_ref()
        .ok_or_else(|| super::errors::NodeNoMemberAddress::from(target))?
        .address
        .member;
    let args = serde_json::json!({
        "node": address,
    });

    let kind = ACTION_KIND_CLUSTER_JOIN.to_string();
    let spec = naction_spec(declaration.approval, args, new_node, kind);
    Ok(spec)
}

/// Common logic to create an [`NActionSpec`] record regardless of expansion mode.
fn naction_spec(
    approval: ActionApproval,
    args: serde_json::Value,
    exec_on: &Node,
    kind: String,
) -> NActionSpec {
    NActionSpec {
        ns_id: exec_on.ns_id.clone(),
        cluster_id: exec_on.cluster_id.clone(),
        node_id: exec_on.node_id.clone(),
        action_id: None,
        args,
        approval,
        kind,
        metadata: Default::default(),
    }
}
