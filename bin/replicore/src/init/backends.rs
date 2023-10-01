//! Dependency backends configuration and initialisation logic.
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use replicore_events::emit::EventsBackendFactory;

/// Error looking for a specific backend implementation.
#[derive(Debug, thiserror::Error)]
pub enum BackendNotFound {
    /// Events backend not recognised.
    #[error("events backend '{0}' not recognised")]
    // (id,)
    Events(String),
}

impl BackendNotFound {
    /// Events backend not recognised.
    pub fn events(id: &str) -> Self {
        Self::Events(id.to_string())
    }
}

/// Registers of backend factories for implementations supported by the process/build.
#[derive(Clone, Default)]
pub struct Backends {
    // Supported Events Platform backends.
    events: HashMap<String, Arc<dyn EventsBackendFactory>>,
}

impl Backends {
    /// Lookup an [`EventsBackendFactory`] by ID.
    pub fn events(&self, id: &str) -> Result<&dyn EventsBackendFactory> {
        let factory = self
            .events
            .get(id)
            .ok_or_else(|| BackendNotFound::events(id))?;
        Ok(factory.as_ref())
    }

    /// Register a new factory for an Events Platform implementation.
    ///
    /// # Panics
    ///
    /// This method panics if the identifier of the new Events Platform backend is already in use.
    pub fn register_events<B, S>(&mut self, id: S, backend: B) -> &mut Self
    where
        B: EventsBackendFactory + 'static,
        S: Into<String>,
    {
        match self.events.entry(id.into()) {
            Entry::Occupied(entry) => {
                panic!(
                    "an EventsBackend with id '{}' is already registered",
                    entry.key()
                )
            }
            Entry::Vacant(entry) => entry.insert(Arc::new(backend)),
        };
        self
    }
}
