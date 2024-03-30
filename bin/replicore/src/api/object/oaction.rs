//! API endpoints for handling `OAction` objects.
use actix_web::web::Data;
use actix_web::web::Path;
use actix_web::web::Query;
use actix_web::HttpResponse;
use futures_util::TryStreamExt;
use uuid::Uuid;

use replisdk::core::models::api::OActionEntry;
use replisdk::core::models::api::OActionList;

use replicore_context::Context;
use replicore_injector::Injector;

use crate::api::Error;

#[derive(Debug, serde::Deserialize)]
struct ListQueryArgs {
    /// Include finished actions in the actions list.
    all: bool,
}

/// Get a OAction object by namespace and name.
#[actix_web::get("/object/replicante.io/v0/oaction/{namespace}/{cluster}/{action}")]
pub async fn get(
    context: Context,
    injector: Data<Injector>,
    path: Path<(String, String, Uuid)>,
) -> Result<HttpResponse, Error> {
    let (ns_id, cluster_id, action_id) = path.into_inner();
    let id = replicore_store::ids::OActionID {
        ns_id,
        cluster_id,
        action_id,
    };
    let query = replicore_store::query::LookupOAction(id);
    let oaction = injector.store.query(&context, query).await?;
    match oaction {
        None => Ok(crate::api::not_found()),
        Some(oaction) => Ok(HttpResponse::Ok().json(oaction)),
    }
}

/// List information about `OAction`s for a cluster.
#[actix_web::get("/list/replicante.io/v0/oaction/{namespace}/{cluster}")]
pub async fn list(
    context: Context,
    injector: Data<Injector>,
    path: Path<(String, String)>,
    query: Query<ListQueryArgs>,
) -> Result<HttpResponse, Error> {
    let (ns_id, cluster_id) = path.into_inner();
    let mut search = replicore_store::query::ListOActions::by(ns_id, cluster_id);
    if query.all {
        search = search.with_finished();
    }

    // Run the search.
    let items = injector.store.query(&context, search).await?;
    let items: Vec<OActionEntry> = items.try_collect().await?;
    let response = OActionList { items };
    let response = serde_json::json!(response);
    Ok(HttpResponse::Ok().json(response))
}
