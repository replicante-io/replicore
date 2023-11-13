//! Home of the [`EventsNull`] backend implementation.
use anyhow::Result;

use replicore_context::Context;
use replicore_events::emit::EventsBackend;
use replicore_events::Event;

/// Events emit implementation that drops all events.
///
/// Intended for use in the `replicore sync` command to configure dependencies
/// while a real events emitter services is not available.
pub struct EventsNull;

#[async_trait::async_trait]
impl EventsBackend for EventsNull {
    async fn audit(&self, _: &Context, _: Event) -> Result<()> {
        Ok(())
    }

    async fn change(&self, _: &Context, _: Event) -> Result<()> {
        Ok(())
    }
}
