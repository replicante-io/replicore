//! Apply API for action objects.
use actix_web::HttpResponse;

use replisdk::core::models::api::NActionSpec;
use replisdk::core::models::api::OActionSpec;

use super::decode;
use super::NACTION_SCHEMA;
use super::OACTION_SCHEMA;
use crate::api::apply::ApplyArgs;
use crate::api::Error;

/// Apply a node action object.
pub async fn naction(args: ApplyArgs<'_>) -> Result<HttpResponse, crate::api::Error> {
    // Verify & decode the cluster spec.
    NACTION_SCHEMA
        .validate(args.object)
        .map_err(crate::api::format_json_schema_errors)?;
    let spec = args.object.get("spec").unwrap().clone();
    let spec: NActionSpec = decode(spec)?;

    // Check the namespace & cluster exist before appling the object.
    super::namespace::check(&args, &spec.ns_id).await?;
    super::cluster::check(&args, &spec.ns_id, &spec.cluster_id).await?;

    // Process the action spec and store it in the DB.
    let sdk = replicore_sdk::CoreSDK::from(args.injector.as_ref());
    let action = sdk.naction_create(&args.context, spec).await;
    let action = match action {
        Err(error) if error.is::<replicore_sdk::errors::NActionExists>() => {
            let error = Error::with_status(actix_web::http::StatusCode::BAD_REQUEST, error);
            return Err(error);
        }
        Err(error) => return Err(error.into()),
        Ok(action) => action,
    };

    // Return the action reference.
    let response = HttpResponse::Ok().json(serde_json::json!(action));
    Ok(response)
}

/// Apply an orchestrator action object.
pub async fn oaction(args: ApplyArgs<'_>) -> Result<HttpResponse, crate::api::Error> {
    // Verify & decode the cluster spec.
    OACTION_SCHEMA
        .validate(args.object)
        .map_err(crate::api::format_json_schema_errors)?;
    let spec = args.object.get("spec").unwrap().clone();
    let spec: OActionSpec = decode(spec)?;

    // Check the namespace & cluster exist before appling the object.
    super::namespace::check(&args, &spec.ns_id).await?;
    super::cluster::check(&args, &spec.ns_id, &spec.cluster_id).await?;

    // Process the action spec and store it in the DB.
    let sdk = replicore_sdk::CoreSDK::from(args.injector.as_ref());
    let action = sdk.oaction_create(&args.context, spec).await;
    let action = match action {
        Err(error) if error.is::<replicore_sdk::errors::OActionExists>() => {
            let error = Error::with_status(actix_web::http::StatusCode::BAD_REQUEST, error);
            return Err(error);
        }
        Err(error) => return Err(error.into()),
        Ok(action) => action,
    };

    // Return the action reference.
    let response = HttpResponse::Ok().json(serde_json::json!(action));
    Ok(response)
}
