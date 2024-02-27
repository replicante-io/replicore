use opentracingrust::Span;

use replicante_models_core::events::Event;
use replicante_models_core::events::Payload;

mod action;
mod cluster;

use crate::follower::Follower;
use crate::Result;

/// Extract information from events and persist them in the view store.
pub fn process(follower: &Follower, event: &Event, span: Option<&mut Span>) -> Result<()> {
    match &event.payload {
        Payload::Action(event) => action::process(follower, event, span),
        Payload::Cluster(event) => cluster::process(follower, event, span),
        _ => Ok(()),
    }
}
