use anyhow::Context;
use anyhow::Result;

use replicante_agent_client::Client;
use replicante_models_core::agent::Agent;
use replicante_models_core::agent::AgentInfo;
use replicante_models_core::agent::AgentStatus;
use replicante_models_core::agent::Node;
use replicante_models_core::events::Event;

use super::emit_event;
use crate::errors::OrchestratorEnder;
use crate::errors::SyncError;
use crate::ClusterOrchestrate;
use crate::ClusterOrchestrateMut;

/// Derive an `AgentStatus` from the result of previous agent interactions.
pub fn sync_agent(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterOrchestrateMut,
    node_id: &str,
    agent_info: Result<()>,
    node_info: Result<()>,
) -> Result<()> {
    // Propagate orchestration ending errors and abort now.
    let agent_info = agent_info.orchestration_failed()?;
    let node_info = node_info.orchestration_failed()?;

    // Determine the agent status based on responses from agents.
    let agent_status = if let Err(ref error) = agent_info {
        AgentStatus::AgentDown(error.to_string())
    } else if let Err(ref error) = node_info {
        AgentStatus::NodeDown(error.to_string())
    } else {
        AgentStatus::Up
    };

    // Handle the overall agent record.
    let agent = Agent::new(
        data.cluster_view.cluster_id.to_string(),
        node_id.to_string(),
        agent_status,
    );
    data_mut
        .new_cluster_view
        .agent(agent.clone())
        .with_context(|| {
            SyncError::cluster_view_update(
                &data.namespace.ns_id,
                &data.cluster_view.cluster_id,
                node_id,
            )
        })?;
    let old = data.cluster_view.agents.get(&agent.host).cloned();
    let result = match old {
        None => agent_new(data, data_mut, node_id, agent),
        Some(old) => agent_update(data, data_mut, node_id, old, agent),
    };

    // Propagate any non-ending errors we may have seen so far.
    result?;
    agent_info?;
    node_info?;
    Ok(())
}

/// Sync `AgentInfo` between the node and core.
pub fn sync_agent_info(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterOrchestrateMut,
    client: &dyn Client,
    node_id: &str,
) -> Result<()> {
    // Grab agent information from the node.
    let span_context = data_mut.span.as_ref().map(|span| span.context().clone());
    let info = client
        .agent_info(span_context)
        .map_err(failure::Fail::compat)
        .with_context(|| {
            SyncError::client_response(
                &data.namespace.ns_id,
                &data.cluster_view.cluster_id,
                node_id,
                "agent-info",
            )
        })?;
    let info = AgentInfo::new(data.cluster_view.cluster_id.clone(), node_id, info);
    data_mut
        .new_cluster_view
        .agent_info(info.clone())
        .with_context(|| {
            SyncError::cluster_view_update(
                &data.namespace.ns_id,
                &data.cluster_view.cluster_id,
                node_id,
            )
        })?;

    // Check if this is a new or existing node and update records respectively.
    let old = data.cluster_view.agents_info.get(&info.host).cloned();
    match old {
        None => agent_info_new(data, data_mut, node_id, info),
        Some(old) => agent_info_update(data, data_mut, node_id, old, info),
    }
}

/// Sync `Node` between the node and core.
pub fn sync_node_info(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterOrchestrateMut,
    client: &dyn Client,
    node_id: &str,
) -> Result<()> {
    // Grab node information from the node.
    let span_context = data_mut.span.as_ref().map(|span| span.context().clone());
    let info = client
        .datastore_info(span_context)
        .map_err(failure::Fail::compat)
        .with_context(|| {
            SyncError::client_response(
                &data.namespace.ns_id,
                &data.cluster_view.cluster_id,
                node_id,
                "datastore-info",
            )
        })?;
    let node = Node::new(info);
    data_mut
        .new_cluster_view
        .node(node.clone())
        .with_context(|| {
            SyncError::cluster_view_update(
                &data.namespace.ns_id,
                &data.cluster_view.cluster_id,
                node_id,
            )
        })?;

    // Check if this is a new or existing node and update records respectively.
    let old = data.cluster_view.node(&node.node_id).cloned();
    match old {
        None => node_info_new(data, data_mut, node_id, node),
        Some(old) => node_info_update(data, data_mut, node_id, old, node),
    }
}

/// Emit the event and persist the new Agent.
fn agent_emit_and_persist<E>(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterOrchestrateMut,
    node_id: &str,
    event: E,
    new: Agent,
) -> Result<()>
where
    E: Into<Option<Event>>,
{
    let span_context = data_mut.span.as_ref().map(|span| span.context().clone());
    if let Some(event) = event.into() {
        emit_event(data, data_mut, node_id, event)?;
    }
    data.store
        .persist()
        .agent(new, span_context)
        .map_err(failure::Fail::compat)
        .with_context(|| {
            SyncError::store_persist_for_node(
                &data.namespace.ns_id,
                &data.cluster_view.cluster_id,
                node_id,
            )
        })
        .map_err(anyhow::Error::from)
}

/// Emit the event and persist the new AgentInfo.
fn agent_info_emit_and_persist(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterOrchestrateMut,
    node_id: &str,
    event: Event,
    new: AgentInfo,
) -> Result<()> {
    let span_context = data_mut.span.as_ref().map(|span| span.context().clone());
    emit_event(data, data_mut, node_id, event)?;
    data.store
        .persist()
        .agent_info(new, span_context)
        .map_err(failure::Fail::compat)
        .with_context(|| {
            SyncError::store_persist_for_node(
                &data.namespace.ns_id,
                &data.cluster_view.cluster_id,
                node_id,
            )
        })
        .map_err(anyhow::Error::from)
}

/// Persist a new AgentInfo record.
fn agent_info_new(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterOrchestrateMut,
    node_id: &str,
    new: AgentInfo,
) -> Result<()> {
    let event = Event::builder().agent().new_agent_info(new.clone());
    agent_info_emit_and_persist(data, data_mut, node_id, event, new)
}

/// Persist an update to an existing `AgentInfo`.
fn agent_info_update(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterOrchestrateMut,
    node_id: &str,
    old: AgentInfo,
    new: AgentInfo,
) -> Result<()> {
    if new == old {
        return Ok(());
    }
    let event = Event::builder().agent().info_changed(old, new.clone());
    agent_info_emit_and_persist(data, data_mut, node_id, event, new)
}

/// Persist a new `Agent` record.
fn agent_new(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterOrchestrateMut,
    node_id: &str,
    new: Agent,
) -> Result<()> {
    let event = Event::builder().agent().new_agent(new.clone());
    agent_emit_and_persist(data, data_mut, node_id, event, new)
}

/// Persist a update to an existing `Agent` record.
fn agent_update(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterOrchestrateMut,
    node_id: &str,
    old: Agent,
    new: Agent,
) -> Result<()> {
    if new == old {
        return Ok(());
    }
    let event = if new.status != old.status {
        Some(Event::builder().agent().transition(old, new.clone()))
    } else {
        None
    };
    agent_emit_and_persist(data, data_mut, node_id, event, new)
}

/// Emit the event and persist the new Node.
fn node_info_emit_and_persist(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterOrchestrateMut,
    node_id: &str,
    event: Event,
    new: Node,
) -> Result<()> {
    let span_context = data_mut.span.as_ref().map(|span| span.context().clone());
    emit_event(data, data_mut, node_id, event)?;
    data.store
        .persist()
        .node(new, span_context)
        .map_err(failure::Fail::compat)
        .with_context(|| {
            SyncError::store_persist_for_node(
                &data.namespace.ns_id,
                &data.cluster_view.cluster_id,
                node_id,
            )
        })
        .map_err(anyhow::Error::from)
}

/// Persist a new `Node` record.
fn node_info_new(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterOrchestrateMut,
    node_id: &str,
    new: Node,
) -> Result<()> {
    let event = Event::builder().node().new_node(new.clone());
    node_info_emit_and_persist(data, data_mut, node_id, event, new)
}

/// Persist an update to an existing `Node`.
fn node_info_update(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterOrchestrateMut,
    node_id: &str,
    old: Node,
    new: Node,
) -> Result<()> {
    if new == old {
        return Ok(());
    }
    let event = Event::builder().node().changed(old, new.clone());
    node_info_emit_and_persist(data, data_mut, node_id, event, new)
}
