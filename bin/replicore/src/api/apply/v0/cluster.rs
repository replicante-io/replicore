//! Apply API for cluster objects.
use actix_web::HttpResponse;

use replisdk::core::models::cluster::ClusterSpec;

use replicore_events::Event;
use replicore_store::query::LookupClusterSpec;

use super::decode;
use super::CLUSTER_SPEC_SCHEMA;
use crate::api::apply::constants::APPLY_CLUSTER_SPEC;
use crate::api::apply::ApplyArgs;

/// Check the persistent store for cluster spec existence.
///
/// If the required cluster spec does not exist return an error for the API client.
pub async fn check(
    args: &ApplyArgs<'_>,
    ns_id: &str,
    cluster_id: &str,
) -> Result<(), crate::api::Error> {
    let query = LookupClusterSpec::by(ns_id, cluster_id);
    let cluster = args.injector.store.query(&args.context, query).await?;
    if cluster.is_some() {
        return Ok(());
    }
    let source = anyhow::anyhow!("ClusterSpec '{ns_id}.{cluster_id}' does not exist");
    let error = crate::api::Error::with_status(actix_web::http::StatusCode::BAD_REQUEST, source);
    Err(error)
}

/// Apply a cluster spec object.
pub async fn cluster_spec(args: ApplyArgs<'_>) -> Result<HttpResponse, crate::api::Error> {
    // Verify & decode the cluster spec.
    CLUSTER_SPEC_SCHEMA
        .validate(args.object)
        .map_err(crate::api::format_json_schema_errors)?;
    let spec = args.object.get("spec").unwrap().clone();
    let cluster: ClusterSpec = decode(spec)?;

    // Ensure the declarative options are valid.
    if let Some(declaration) = &cluster.declaration.definition {
        if declaration.cluster_id != cluster.cluster_id {
            let source = anyhow::anyhow!(
                "ClusterSpec.declaration.definition.cluster_id does not match ClusterSpec.cluster_id"
            );
            return Err(crate::api::Error::with_status(
                actix_web::http::StatusCode::BAD_REQUEST,
                source,
            ));
        }
    }
    if cluster.declaration.definition.is_some() && cluster.platform.is_none() {
        let source = anyhow::anyhow!(
            "ClusterSpec.platform MUST be set when ClusterSpec.declaration.definition is set"
        );
        return Err(crate::api::Error::with_status(
            actix_web::http::StatusCode::BAD_REQUEST,
            source,
        ));
    }

    // Check the namespace exists before appling the object.
    super::namespace::check(&args, &cluster.ns_id).await?;

    // Apply the cluster spec.
    let event = Event::new_with_payload(APPLY_CLUSTER_SPEC, &cluster)?;
    args.injector.events.change(&args.context, event).await?;
    args.injector.store.persist(&args.context, cluster).await?;
    Ok(crate::api::done())
}
