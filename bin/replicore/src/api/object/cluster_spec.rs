//! API endpoints for handling `ClusterSpec` objects.
use actix_web::web::Data;
use actix_web::web::Path;
use actix_web::HttpResponse;
use futures_util::TryStreamExt;

use replisdk::core::models::api::ClusterSpecEntry;
use replisdk::core::models::api::ClusterSpecList;

use replicore_cluster_view::ClusterView;
use replicore_context::Context;
use replicore_events::Event;
use replicore_injector::Injector;

use crate::api::constants::CLUSTER_SPEC_DELETED;
use crate::api::Error;

/// Delete a ClusterSpec object from a namespace.
#[actix_web::delete("/object/replicante.io/v0/clusterspec/{namespace}/{name}")]
pub async fn delete(
    context: Context,
    injector: Data<Injector>,
    path: Path<(String, String)>,
) -> Result<HttpResponse, Error> {
    let (ns_id, name) = path.into_inner();
    let id = replicore_store::ids::NamespacedResourceID { ns_id, name };
    let event = Event::new_with_payload(CLUSTER_SPEC_DELETED, &id)?;
    let op = replicore_store::delete::DeleteClusterSpec(id);
    injector.events.change(&context, event).await?;
    injector.store.delete(&context, op).await?;
    Ok(crate::api::done())
}

/// Get a ClusterDiscovery object by namespace and name.
#[actix_web::get("/object/replicante.io/v0/clusterspec/{namespace}/{name}/discovery")]
pub async fn discovery(
    context: Context,
    injector: Data<Injector>,
    path: Path<(String, String)>,
) -> Result<HttpResponse, Error> {
    let (ns_id, name) = path.into_inner();
    let id = replicore_store::ids::NamespacedResourceID { ns_id, name };
    let query = replicore_store::query::LookupClusterDiscovery(id);
    let cluster_disc = injector.store.query(&context, query).await?;
    match cluster_disc {
        None => Ok(crate::api::not_found()),
        Some(cluster_disc) => Ok(HttpResponse::Ok().json(cluster_disc)),
    }
}

/// Get a ClusterSpec object by namespace and name.
#[actix_web::get("/object/replicante.io/v0/clusterspec/{namespace}/{name}")]
pub async fn get(
    context: Context,
    injector: Data<Injector>,
    path: Path<(String, String)>,
) -> Result<HttpResponse, Error> {
    let (ns_id, name) = path.into_inner();
    let id = replicore_store::ids::NamespacedResourceID { ns_id, name };
    let query = replicore_store::query::LookupClusterSpec(id);
    let cluster_spec = injector.store.query(&context, query).await?;
    match cluster_spec {
        None => Ok(crate::api::not_found()),
        Some(cluster_spec) => Ok(HttpResponse::Ok().json(cluster_spec)),
    }
}

/// List information about `ClusterSpec`s stored in a namespace.
#[actix_web::get("/list/replicante.io/v0/clusterspec/{namespace}")]
pub async fn list(
    context: Context,
    injector: Data<Injector>,
    path: Path<String>,
) -> Result<HttpResponse, Error> {
    let ns_id = replicore_store::ids::NamespaceID {
        id: path.into_inner(),
    };
    let query = replicore_store::query::ListClusterSpecs(ns_id);
    let items = injector.store.query(&context, query).await?;
    let items: Vec<ClusterSpecEntry> = items.try_collect().await?;
    let response = ClusterSpecList { items };
    let response = serde_json::json!(response);
    Ok(HttpResponse::Ok().json(response))
}

/// Submit a cluster orchestration task for background execution.
#[actix_web::post("/object/replicante.io/v0/clusterspec/{namespace}/{name}/orchestrate")]
pub async fn orchestrate(
    context: Context,
    injector: Data<Injector>,
    path: Path<(String, String)>,
) -> Result<HttpResponse, Error> {
    let (ns_id, cluster_id) = path.into_inner();
    let task = replicore_task_orchestrate::OrchestrateCluster::new(ns_id, cluster_id);
    injector.tasks.submit(&context, task).await?;
    Ok(crate::api::done())
}

/// Get an [`OrchestrateReport`] by cluster namespace and name.
#[actix_web::get("/object/replicante.io/v0/clusterspec/{namespace}/{name}/orchestrate/report")]
pub async fn orchestrate_report(
    context: Context,
    injector: Data<Injector>,
    path: Path<(String, String)>,
) -> Result<HttpResponse, Error> {
    let (ns_id, name) = path.into_inner();
    let id = replicore_store::ids::NamespacedResourceID { ns_id, name };
    let query = replicore_store::query::LookupOrchestrateReport(id);
    let report = injector.store.query(&context, query).await?;
    match report {
        None => Ok(crate::api::not_found()),
        Some(report) => Ok(HttpResponse::Ok().json(report)),
    }
}

/// Get a [`ClusterView`] by cluster namespace and name.
#[actix_web::get("/object/replicante.io/v0/clusterspec/{namespace}/{name}/view")]
pub async fn view(
    context: Context,
    injector: Data<Injector>,
    path: Path<(String, String)>,
) -> Result<HttpResponse, Error> {
    let (ns_id, name) = path.into_inner();
    let id = replicore_store::ids::NamespacedResourceID { ns_id, name };
    let query = replicore_store::query::LookupClusterSpec(id);
    let spec = match injector.store.query(&context, query).await? {
        None => return Ok(crate::api::not_found()),
        Some(spec) => spec,
    };
    let view = ClusterView::load(&context, &injector.store, spec)
        .await?
        .finish();
    Ok(HttpResponse::Ok().json(view))
}
