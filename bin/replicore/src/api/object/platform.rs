//! API endpoints for handling `Platform` objects.
use actix_web::web::Data;
use actix_web::web::Path;
use actix_web::HttpResponse;
use futures_util::TryStreamExt;

use replisdk::core::models::api::PlatformEntry;
use replisdk::core::models::api::PlatformList;

use replicore_context::Context;
use replicore_events::Event;
use replicore_injector::Injector;

use crate::api::constants::PLATFORM_DELETED;
use crate::api::Error;

/// Delete a `Platform` object from a namespace.
#[actix_web::delete("/object/replicante.io/v0/platform/{namespace}/{name}")]
pub async fn delete(
    context: Context,
    injector: Data<Injector>,
    path: Path<(String, String)>,
) -> Result<HttpResponse, Error> {
    let (ns_id, name) = path.into_inner();
    let id = replicore_store::ids::NamespacedResourceID { ns_id, name };
    let event = Event::new_with_payload(PLATFORM_DELETED, &id)?;
    let op = replicore_store::delete::DeletePlatform(id);
    injector.events.change(&context, event).await?;
    injector.store.delete(&context, op).await?;
    Ok(crate::api::done())
}

/// Submit a platform discovery task for background execution.
#[actix_web::get("/object/replicante.io/v0/platform/{namespace}/{name}/discover")]
pub async fn discover(
    context: Context,
    injector: Data<Injector>,
    path: Path<(String, String)>,
) -> Result<HttpResponse, Error> {
    let (ns_id, name) = path.into_inner();
    let task = replicore_task_discovery::DiscoverPlatform { ns_id, name };
    injector.tasks.submit(&context, task).await?;
    Ok(crate::api::done())
}

/// Get a `Platform` object by namespace and name.
#[actix_web::get("/object/replicante.io/v0/platform/{namespace}/{name}")]
pub async fn get(
    context: Context,
    injector: Data<Injector>,
    path: Path<(String, String)>,
) -> Result<HttpResponse, Error> {
    let (ns_id, name) = path.into_inner();
    let id = replicore_store::ids::NamespacedResourceID { ns_id, name };
    let query = replicore_store::query::LookupPlatform(id);
    let platform = injector.store.query(&context, query).await?;
    match platform {
        None => Ok(crate::api::not_found()),
        Some(platform) => Ok(HttpResponse::Ok().json(platform)),
    }
}

/// List information about `Platform`s configured in a namespace.
#[actix_web::get("/list/replicante.io/v0/platform/{namespace}")]
pub async fn list(
    context: Context,
    injector: Data<Injector>,
    path: Path<String>,
) -> Result<HttpResponse, Error> {
    let ns_id = replicore_store::ids::NamespaceID {
        id: path.into_inner(),
    };
    let query = replicore_store::query::ListPlatforms(ns_id);
    let items = injector.store.query(&context, query).await?;
    let items: Vec<PlatformEntry> = items.try_collect().await?;
    let response = PlatformList { items };
    let response = serde_json::json!(response);
    Ok(HttpResponse::Ok().json(response))
}
