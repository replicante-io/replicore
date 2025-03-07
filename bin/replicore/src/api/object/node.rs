//! API endpoints for handling `Node` objects.
use actix_web::web::Data;
use actix_web::web::Path;
use actix_web::HttpResponse;

use replisdk::core::models::node::NodeStatus;

use replicore_context::Context;
use replicore_events::Event;
use replicore_injector::Injector;

use crate::api::constants::NODE_UPDATE_API;
use crate::api::Error;

/// Mark a cluster node for deletion.
#[actix_web::delete("/object/replicante.io/v0/clusterspec/{namespace}/{name}/{node}")]
pub async fn delete(
    context: Context,
    injector: Data<Injector>,
    path: Path<(String, String, String)>,
) -> Result<HttpResponse, Error> {
    let (ns_id, name, node) = path.into_inner();
    let id = replicore_store::ids::NodeID::by(ns_id, name, node);
    let op = replicore_store::query::LookupClusterNode(id);
    let mut node = match injector.store.query(&context, op).await? {
        None => return Ok(crate::api::done()),
        Some(node) if node.node_status.is_deleting() => return Ok(crate::api::done()),
        Some(node) => node,
    };

    node.node_status = NodeStatus::Deleting;
    let event = Event::new_with_payload(NODE_UPDATE_API, &node)?;
    injector.events.change(&context, event).await?;
    injector.store.persist(&context, node).await?;
    Ok(crate::api::done())
}
