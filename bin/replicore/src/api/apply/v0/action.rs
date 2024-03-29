//! Apply API for action objects.
use actix_web::HttpResponse;

use replisdk::core::models::api::OActionSpec;
use replisdk::core::models::oaction::OAction;
use replisdk::core::models::oaction::OActionState;

use replicore_events::Event;
use replicore_store::query::LookupOAction;

use super::decode;
use super::OACTION_SCHEMA;
use crate::api::apply::constants::APPLY_OACTION;
use crate::api::apply::ApplyArgs;

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

    // If an Action ID is set ensure it does not exist.
    if let Some(action_id) = spec.action_id {
        let query = LookupOAction::by(&spec.ns_id, &spec.cluster_id, action_id);
        let oaction = args.injector.store.query(&args.context, query).await?;
        if oaction.is_some() {
            let ns_id = &spec.ns_id;
            let cluster_id = &spec.cluster_id;
            let source = anyhow::anyhow!(
                "OAction '{action_id}' already exists for cluster '{ns_id}.{cluster_id}'"
            );
            let error = crate::api::Error::with_status(
                actix_web::http::StatusCode::BAD_REQUEST,
                source,
            );
            return Err(error);
        }
    }

    // Expand the spec into a full object.
    let approved = args
        .object
        .get("approved")
        .and_then(|approved| approved.as_bool())
        .unwrap_or(true);
    let state = match approved {
        false => OActionState::PendingApprove,
        true => OActionState::PendingSchedule,
    };
    let oaction = OAction {
        ns_id: spec.ns_id,
        cluster_id: spec.cluster_id,
        action_id: spec.action_id.unwrap_or_else(uuid::Uuid::new_v4),
        args: spec.args,
        created_ts: time::OffsetDateTime::now_utc(),
        finished_ts: None,
        kind: spec.kind,
        metadata: spec.metadata,
        scheduled_ts: None,
        state,
        state_payload: None,
        state_payload_error: None,
        timeout: spec.timeout,
    };

    // Apply the cluster spec.
    let event = Event::new_with_payload(APPLY_OACTION, &oaction)?;
    args.injector.events.change(&args.context, event).await?;
    args.injector.store.persist(&args.context, oaction).await?;
    Ok(crate::api::done())
}
