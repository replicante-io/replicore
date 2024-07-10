//! API endpoints for handling `NAction` objects.
use actix_web::web::Data;
use actix_web::web::Path;
use actix_web::web::Query;
use actix_web::HttpResponse;
use futures_util::TryStreamExt;
use uuid::Uuid;

use replisdk::core::models::api::NActionEntry;
use replisdk::core::models::api::NActionList;
use replisdk::core::models::naction::NActionPhase;

use replicore_context::Context;
use replicore_injector::Injector;

use crate::api::Error;

#[derive(Debug, serde::Deserialize)]
struct ListQueryArgs {
    /// Include finished actions in the actions list.
    all: bool,

    /// Filter actions to list by node.
    node_id: Option<String>,
}

/// Approve an NAction object for scheduling.
#[actix_web::post("/object/replicante.io/v0/naction/{namespace}/{cluster}/{node}/{action}/approve")]
pub async fn approve(
    context: Context,
    injector: Data<Injector>,
    path: Path<(String, String, String, Uuid)>,
) -> Result<HttpResponse, Error> {
    // Load the action record.
    let (ns_id, cluster_id, node_id, action_id) = path.into_inner();
    let id = replicore_store::ids::NActionID {
        ns_id,
        cluster_id,
        node_id,
        action_id,
    };
    let query = replicore_store::query::LookupNAction(id);
    let action = injector.store.query(&context, query).await?;
    let action = match action {
        None => return Ok(crate::api::not_found()),
        Some(action) => action,
    };

    // Verify the action can be approved.
    if !matches!(action.state.phase, NActionPhase::PendingApprove) {
        let source = anyhow::anyhow!("only PENDING_APPROVE nactions can be approved");
        return Err(Error::bad_request(source));
    }

    // Update record and emit events.
    let sdk = replicore_sdk::CoreSDK::from(injector.as_ref());
    sdk.naction_approve(&context, action).await?;

    // Update the record in the store.
    Ok(crate::api::done())
}

/// Cancel a node action and prevent any further execution (including running actions).
#[actix_web::post("/object/replicante.io/v0/naction/{namespace}/{cluster}/{node}/{action}/cancel")]
pub async fn cancel(
    context: Context,
    injector: Data<Injector>,
    path: Path<(String, String, String, Uuid)>,
) -> Result<HttpResponse, Error> {
    // Load the action record.
    let (ns_id, cluster_id, node_id, action_id) = path.into_inner();
    let id = replicore_store::ids::NActionID {
        ns_id,
        cluster_id,
        node_id,
        action_id,
    };
    let query = replicore_store::query::LookupNAction(id);
    let action = injector.store.query(&context, query).await?;
    let action = match action {
        None => return Ok(crate::api::not_found()),
        Some(action) => action,
    };

    // Verify the action can be approved.
    if action.state.phase.is_final() {
        let source = anyhow::anyhow!("cannot cancel a finished action");
        return Err(Error::bad_request(source));
    }

    // Update record and emit events.
    let sdk = replicore_sdk::CoreSDK::from(injector.as_ref());
    sdk.naction_cancel(&context, action).await?;

    // Update the record in the store.
    Ok(crate::api::done())
}

/// Get a NAction object by namespace and name.
#[actix_web::get("/object/replicante.io/v0/naction/{namespace}/{cluster}/{node}/{action}")]
pub async fn get(
    context: Context,
    injector: Data<Injector>,
    path: Path<(String, String, String, Uuid)>,
) -> Result<HttpResponse, Error> {
    let (ns_id, cluster_id, node_id, action_id) = path.into_inner();
    let id = replicore_store::ids::NActionID {
        ns_id,
        cluster_id,
        node_id,
        action_id,
    };
    let query = replicore_store::query::LookupNAction(id);
    let action = injector.store.query(&context, query).await?;
    match action {
        None => Ok(crate::api::not_found()),
        Some(action) => Ok(HttpResponse::Ok().json(action)),
    }
}

/// List information about `NAction`s for a cluster.
#[actix_web::get("/list/replicante.io/v0/naction/{namespace}/{cluster}")]
pub async fn list(
    context: Context,
    injector: Data<Injector>,
    path: Path<(String, String)>,
    query: Query<ListQueryArgs>,
) -> Result<HttpResponse, Error> {
    let (ns_id, cluster_id) = path.into_inner();
    let mut search = replicore_store::query::ListNActions::by(ns_id, cluster_id);
    if query.all {
        search = search.with_finished();
    }
    if let Some(node_id) = &query.node_id {
        search = search.with_node(node_id.clone());
    }

    // Run the search.
    let items = injector.store.query(&context, search).await?;
    let items: Vec<NActionEntry> = items.try_collect().await?;
    let response = NActionList { items };
    let response = serde_json::json!(response);
    Ok(HttpResponse::Ok().json(response))
}

/// Reject a NAction object to prevent scheduling.
#[actix_web::post("/object/replicante.io/v0/naction/{namespace}/{cluster}/{node}/{action}/reject")]
pub async fn reject(
    context: Context,
    injector: Data<Injector>,
    path: Path<(String, String, String, Uuid)>,
) -> Result<HttpResponse, Error> {
    // Load the action record.
    let (ns_id, cluster_id, node_id, action_id) = path.into_inner();
    let id = replicore_store::ids::NActionID {
        ns_id,
        cluster_id,
        node_id,
        action_id,
    };
    let query = replicore_store::query::LookupNAction(id);
    let action = injector.store.query(&context, query).await?;
    let action = match action {
        None => return Ok(crate::api::not_found()),
        Some(action) => action,
    };

    // Verify the action can be approved.
    if !matches!(action.state.phase, NActionPhase::PendingApprove) {
        let source = anyhow::anyhow!("only PENDING_APPROVE nactions can be rejected");
        return Err(Error::bad_request(source));
    }

    // Update record and emit events.
    let sdk = replicore_sdk::CoreSDK::from(injector.as_ref());
    sdk.naction_reject(&context, action).await?;

    // Update the record in the store.
    Ok(crate::api::done())
}
