use actix_web::HttpResponse;
use failure::ResultExt;
use opentracingrust::Span;
use uuid::Uuid;

use replicante_models_core::actions::node::Action;
use replicante_models_core::events::Event;
use replicante_store_primary::store::Store;
use replicante_stream_events::EmitMessage;
use replicante_stream_events::Stream;

use crate::ErrorKind;
use crate::Result;

pub mod approve;
pub mod disapprove;
pub mod summary;

/// Load a NodeAction, apply a transformation to it and persist it back.
///
/// A NodeAction changed event is emitted for the transformation.
fn load_transform_event_persist<Transform>(
    cluster_id: String,
    action_id: Uuid,
    span: Option<&mut Span>,
    events: &Stream,
    store: &Store,
    transform: Transform,
) -> Result<Option<HttpResponse>>
where
    Transform: FnOnce(Action) -> std::result::Result<Action, HttpResponse>,
{
    let span_context = span.map(|span| span.context().clone());

    // Load the action.
    let action = store
        .action(cluster_id, action_id)
        .get(span_context.clone())
        .with_context(|_| ErrorKind::PrimaryStoreQuery("action"))?;

    // Reject requests for missing actions.
    let action = match action {
        Some(action) => action,
        None => {
            let response = HttpResponse::NotFound().json(serde_json::json!({
                "error": "action not found",
                "layers": [],
            }));
            return Ok(Some(response));
        }
    };

    // Transform (a copy) of the action.
    let old_action = action.clone();
    let action = match transform(action) {
        Err(response) => return Ok(Some(response)),
        Ok(action) => action,
    };
    if old_action == action {
        return Ok(None);
    }

    // Emit the change event.
    let event = Event::builder()
        .action()
        .changed(old_action, action.clone());
    let stream_key = event.entity_id().partition_key();
    let event = EmitMessage::with(stream_key, event)
        .with_context(|_| ErrorKind::EventsStreamEmit("ACTION_CHANGED"))?
        .trace(span_context.clone());
    events
        .emit(event)
        .with_context(|_| ErrorKind::EventsStreamEmit("ACTION_CHANGED"))?;

    // Persist the action back to the store.
    store
        .persist()
        .action(action, span_context)
        .with_context(|_| ErrorKind::PrimaryStorePersist("action"))?;
    Ok(None)
}
