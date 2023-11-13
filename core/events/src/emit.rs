//! Interfaces to emit events to a streaming platform.
use std::sync::Arc;

use anyhow::Result;
use serde_json::Value as Json;

use replicore_context::Context;

use super::Event;

/// Emit events to the backing events streaming platform.
#[derive(Clone)]
pub struct Events(Arc<dyn EventsBackend>);

impl Events {
    /// Emit an auditing event.
    pub async fn audit(&self, context: &Context, event: Event) -> Result<()> {
        self.0.audit(context, event).await
    }

    /// Emit an event about a change to an element in the system.
    pub async fn change(&self, context: &Context, event: Event) -> Result<()> {
        self.0.change(context, event).await
    }
}

impl<T> From<T> for Events
where
    T: EventsBackend + 'static,
{
    fn from(value: T) -> Self {
        Events(Arc::new(value))
    }
}

/// Operations implemented by Event Streaming Platforms supported by Replicante Core.
#[async_trait::async_trait]
pub trait EventsBackend: Send + Sync {
    /// Emit an auditing event.
    async fn audit(&self, context: &Context, event: Event) -> Result<()>;

    /// Emit an event about a change to an element in the system.
    async fn change(&self, context: &Context, event: Event) -> Result<()>;
}

/// Initialisation logic for the event streaming platform and the client to access it.
#[async_trait::async_trait]
pub trait EventsBackendFactory: Send + Sync {
    /// Validate the user provided configuration for the backend.
    fn conf_check(&self, context: &Context, conf: &Json) -> Result<()>;

    /// Register backend specific metrics.
    fn register_metrics(&self, registry: &prometheus::Registry) -> Result<()>;

    /// Instantiate an [`Events`] object to emit events to the streaming platform.
    async fn events<'a>(&self, args: EventsBackendFactoryArgs<'a>) -> Result<Events>;

    /// Synchronise (initialise of migrate) the streaming platform to handle RepliCore [`Event`]s.
    async fn sync<'a>(&self, args: EventsBackendFactorySyncArgs<'a>) -> Result<()>;
}

/// Arguments passed to the [`EventsBackendFactory`] client initialisation method.
pub struct EventsBackendFactoryArgs<'a> {
    /// The configuration block for the backend to initialise.
    pub conf: &'a Json,

    /// Container for operation scoped values.
    pub context: &'a Context,
}

/// Arguments passed to the [`EventsBackendFactory`] client synchronisation method.
pub struct EventsBackendFactorySyncArgs<'a> {
    /// The configuration block for the backend to Synchronise.
    pub conf: &'a Json,

    /// Container for operation scoped values.
    pub context: &'a Context,
}

#[cfg(any(test, feature = "test-fixture"))]
pub use self::fixture::{EventsFixture, EventsFixtureBackend};

#[cfg(any(test, feature = "test-fixture"))]
mod fixture {
    use std::time::Duration;

    use anyhow::Result;
    use tokio::sync::broadcast;
    use tokio::sync::broadcast::Receiver;
    use tokio::sync::broadcast::Sender;

    use replicore_context::Context;

    use super::Event;
    use super::EventsBackend;

    /// Introspection tools for events emitted during unit tests.
    pub struct EventsFixture {
        audit: Receiver<Event>,
        changes: Receiver<Event>,
        send_audit: Sender<Event>,
        send_changes: Sender<Event>,
    }

    impl Clone for EventsFixture {
        fn clone(&self) -> Self {
            let audit = self.send_audit.subscribe();
            let changes = self.send_changes.subscribe();
            Self {
                audit,
                changes,
                send_audit: self.send_audit.clone(),
                send_changes: self.send_changes.clone(),
            }
        }
    }

    impl EventsFixture {
        /// Create a backend that will send events to this fixture.
        pub fn backend(&self) -> EventsFixtureBackend {
            let audit = self.send_audit.clone();
            let changes = self.send_changes.clone();
            EventsFixtureBackend { audit, changes }
        }

        /// Initialise an event streaming backend fixture for unit tests.
        pub fn new() -> EventsFixture {
            let (send_audit, audit) = broadcast::channel(50);
            let (send_changes, changes) = broadcast::channel(50);
            EventsFixture {
                audit,
                changes,
                send_audit,
                send_changes,
            }
        }

        /// Fetch the next [`Event`] emitted onto the audit stream.
        pub async fn pop_audit(&mut self) -> Result<Event> {
            let event = self.audit.recv().await?;
            Ok(event)
        }

        /// Fetch the next [`Event`] emitted onto the audit stream, with a timeout.
        pub async fn pop_audit_timeout(&mut self, timeout: Duration) -> Result<Event> {
            let event = tokio::time::timeout(timeout, self.pop_audit()).await?;
            event
        }

        /// Fetch the next [`Event`] emitted onto the change stream.
        pub async fn pop_change(&mut self) -> Result<Event> {
            let event = self.changes.recv().await?;
            Ok(event)
        }

        /// Fetch the next [`Event`] emitted onto the change stream, with a timeout.
        pub async fn pop_change_timeout(&mut self, timeout: Duration) -> Result<Event> {
            let event = tokio::time::timeout(timeout, self.pop_change()).await?;
            event
        }
    }

    /// Events backend for unit tests.
    pub struct EventsFixtureBackend {
        audit: Sender<Event>,
        changes: Sender<Event>,
    }

    #[async_trait::async_trait]
    impl EventsBackend for EventsFixtureBackend {
        async fn audit(&self, _: &Context, event: Event) -> Result<()> {
            self.audit.send(event)?;
            Ok(())
        }

        async fn change(&self, _: &Context, event: Event) -> Result<()> {
            self.changes.send(event)?;
            Ok(())
        }
    }
}
