//! Apply API for namespaces.
use actix_web::HttpResponse;

use replisdk::core::models::namespace::Namespace;
use replisdk::core::models::namespace::NamespaceStatus;

use replicore_events::Event;
use replicore_store::query::LookupNamespace;

use super::decode;
use super::NAMESPACE_SCHEMA;
use crate::api::apply::constants::APPLY_NAMESPACE;
use crate::api::apply::ApplyArgs;

/// Apply a namespace object.
pub async fn namespace(args: ApplyArgs<'_>) -> Result<HttpResponse, crate::api::Error> {
    // Verify & decode Namespace.
    NAMESPACE_SCHEMA
        .validate(args.object)
        .map_err(crate::api::format_json_schema_errors)?;
    let spec = args.object.get("spec").unwrap().clone();
    let namespace: Namespace = decode(spec)?;

    // For existing namespaces ensure the status transition is valid.
    let lookup = LookupNamespace::from(&namespace);
    let lookup = args.injector.store.query(&args.context, lookup).await?;
    if let Some(lookup) = lookup {
        let from_status = lookup.status;
        let to_status = &namespace.status;
        match from_status {
            NamespaceStatus::Deleted if !matches!(to_status, NamespaceStatus::Deleted) => {
                let error = anyhow::anyhow!("deleted namespaces cannot be change");
                let error = crate::api::Error::bad_request(error);
                return Err(error);
            }
            NamespaceStatus::Deleting if !matches!(to_status, NamespaceStatus::Deleting) => {
                let error = anyhow::anyhow!("deleting namespace status cannot be change");
                let error = crate::api::Error::bad_request(error);
                return Err(error);
            }
            _ => (),
        };
    }

    // Apply the namespace.
    let event = Event::new_with_payload(APPLY_NAMESPACE, &namespace)?;
    args.injector.events.change(&args.context, event).await?;
    args.injector
        .store
        .persist(&args.context, namespace)
        .await?;
    Ok(crate::api::done())
}

/// Check the persistent store for namespace existence.
///
/// If the required namespace does not exist return an error for the API client.
pub async fn check(args: &ApplyArgs<'_>, id: &str) -> Result<(), crate::api::Error> {
    let query = LookupNamespace::from(id);
    let namespace = args.injector.store.query(&args.context, query).await?;
    if namespace.is_some() {
        return Ok(());
    }
    let source = anyhow::anyhow!("Namespace '{id}' does not exist");
    let error = crate::api::Error::with_status(actix_web::http::StatusCode::BAD_REQUEST, source);
    Err(error)
}
