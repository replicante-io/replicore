//! API endpoints for handling persisted namespaces.
use actix_web::web::Data;
use actix_web::web::Path;
use actix_web::HttpResponse;
use futures_util::TryStreamExt;

use replisdk::core::models::api::NamespaceEntry;
use replisdk::core::models::api::NamespaceList;
use replisdk::core::models::namespace::NamespaceStatus;

use replicore_context::Context;
use replicore_events::Event;
use replicore_injector::Injector;

use crate::api::constants::NAMESPACE_DELETE_REQUESTED;
use crate::api::Error;

/// Delete a namespace by ID.
#[actix_web::delete("/object/replicante.io/v0/namespace/{id}")]
pub async fn delete(
    context: Context,
    injector: Data<Injector>,
    path: Path<String>,
) -> Result<HttpResponse, Error> {
    // Find the namespace, succeeding if it does not exist.
    let query = replicore_store::query::LookupNamespace::from(path.as_str());
    let namespace = injector.store.query(&context, query).await?;
    let namespace = match namespace {
        None => return Ok(crate::api::done()),
        Some(namespace) => namespace,
    };

    // Ignore requests for namespaces already deleting.
    let deleting = matches!(
        namespace.status,
        NamespaceStatus::Deleted | NamespaceStatus::Deleting
    );
    if deleting {
        slog::debug!(
            context.logger,
            "Namespace already deleting/deleted, ignoring";
            "namespace" => namespace.id,
        );
        return Ok(crate::api::done());
    }

    // Update namespace to the deleting state.
    let mut namespace = namespace;
    namespace.status = NamespaceStatus::Deleting;
    let event = Event::new_with_payload(NAMESPACE_DELETE_REQUESTED, &namespace)?;
    injector.events.change(&context, event).await?;
    injector.store.persist(&context, namespace).await?;
    Ok(crate::api::done())
}

/// Get a namespace object by ID.
#[actix_web::get("/object/replicante.io/v0/namespace/{id}")]
pub async fn get(
    context: Context,
    injector: Data<Injector>,
    path: Path<String>,
) -> Result<HttpResponse, Error> {
    let query = replicore_store::query::LookupNamespace::from(path.as_str());
    let namespace = injector.store.query(&context, query).await?;
    match namespace {
        None => Ok(crate::api::not_found()),
        Some(namespace) => Ok(HttpResponse::Ok().json(namespace)),
    }
}

/// List the IDs of all namespaces on the control plane.
#[actix_web::get("/list/replicante.io/v0/namespace")]
pub async fn list(context: Context, injector: Data<Injector>) -> Result<HttpResponse, Error> {
    let query = replicore_store::query::ListNamespaces;
    let items = injector.store.query(&context, query).await?;
    let items: Vec<NamespaceEntry> = items.try_collect().await?;
    let response = NamespaceList { items };
    let response = serde_json::json!(response);
    Ok(HttpResponse::Ok().json(response))
}
