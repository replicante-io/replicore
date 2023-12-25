//! Apply API for platform objects.
use actix_web::HttpResponse;

use replisdk::core::models::platform::Platform;

use replicore_events::Event;

use super::decode;
use super::PLATFORM_SCHEMA;
use crate::api::apply::constants::APPLY_PLATFORM;
use crate::api::apply::ApplyArgs;

/// Apply a platform object.
pub async fn platform(args: ApplyArgs<'_>) -> Result<HttpResponse, crate::api::Error> {
    // Verify & decode the platform.
    PLATFORM_SCHEMA
        .validate(args.object)
        .map_err(crate::api::format_json_schema_errors)?;
    let spec = args.object.get("spec").unwrap().clone();
    let platform: Platform = decode(spec)?;

    // Check the namespace exists before appling the object.
    super::namespace::check(&args, &platform.ns_id).await?;

    // Apply the platform.
    let event = Event::new_with_payload(APPLY_PLATFORM, &platform)?;
    args.injector.events.change(&args.context, event).await?;
    args.injector.store.persist(&args.context, platform).await?;
    Ok(crate::api::done())
}
