//! Process and persist node information from agents.
use anyhow::Result;

use replisdk::agent::models::Node as AgentNode;
use replisdk::agent::models::NodeStatus as AgentNodeStatus;
use replisdk::core::models::cluster::ClusterSpec;
use replisdk::core::models::node::Node;
use replisdk::core::models::node::NodeDetails;
use replisdk::core::models::node::NodeStatus;
use replisdk::platform::models::ClusterDiscoveryNode;

use replicore_context::Context;
use replicore_events::Event;

use crate::sync::SyncData;

/// Logic for persisting Node information about cluster nodes.
///
/// - Adds the node to the cluster view builder.
/// - Emits associated events.
/// - Persist node record to the store.
pub async fn persist(context: &Context, data: &SyncData, node: Node) -> Result<()> {
    // Emit node sync event as appropriate.
    let code = match data.cluster_current.nodes.get(&node.node_id) {
        Some(current) if current.as_ref() != &node => Some(crate::constants::NODE_SYNC_UPDATE),
        None => Some(crate::constants::NODE_SYNC_NEW),
        _ => None,
    };
    if let Some(code) = code {
        let event = Event::new_with_payload(code, node.clone())?;
        data.injector.events.change(context, event).await?;
    }

    // Update view and store.
    data.cluster_new_mut().node_info(node.clone())?;
    data.injector.store.persist(context, node).await?;
    Ok(())
}

/// Process agent information to populate a core [`Node`] object.
pub fn process(incomplete: bool, ag_node: AgentNode, mut node: Node) -> Node {
    let incomplete = matches!(ag_node.node_status, AgentNodeStatus::Healthy) && incomplete;
    node.node_status = if incomplete {
        NodeStatus::Incomplete
    } else {
        ag_node.node_status.into()
    };
    node.details = Some(NodeDetails {
        address: ag_node.address,
        agent_version: ag_node.agent_version,
        attributes: ag_node.attributes,
        store_id: ag_node.store_id,
        store_version: ag_node.store_version,
    });
    node
}

/// Initialise a new [`Node`] information record as unreachable.
pub fn unreachable(spec: &ClusterSpec, node: &ClusterDiscoveryNode) -> Node {
    Node {
        ns_id: spec.ns_id.clone(),
        cluster_id: spec.cluster_id.clone(),
        node_id: node.node_id.clone(),
        details: None,
        node_status: NodeStatus::Unreachable,
    }
}
