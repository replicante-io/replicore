//! Dependency backends configuration and initialisation logic.
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use replicore_events::emit::EventsFactory;
use replicore_store::StoreFactory;
use replicore_tasks::factory::TasksFactory;

/// Error looking for a specific backend implementation.
#[derive(Debug, thiserror::Error)]
pub enum BackendNotFound {
    /// Events backend not recognised.
    #[error("events backend '{0}' not recognised")]
    // (id,)
    Events(String),

    /// Persistent Store backend not recognised.
    #[error("persistent store backend '{0}' not recognised")]
    // (id,)
    Store(String),

    /// Background Tasks backend not recognised.
    #[error("background tasks backend '{0}' not recognised")]
    // (id,)
    Tasks(String),
}

impl BackendNotFound {
    /// Events backend not recognised.
    pub fn events(id: &str) -> Self {
        Self::Events(id.to_string())
    }

    /// Persistent Store backend not recognised.
    pub fn store(id: &str) -> Self {
        Self::Store(id.to_string())
    }

    /// Background Tasks backend not recognised.
    pub fn tasks(id: &str) -> Self {
        Self::Tasks(id.to_string())
    }
}

/// Registers of backend factories for implementations supported by the process/build.
#[derive(Clone, Default)]
pub struct Backends {
    // Supported Events Platform backends.
    events: HashMap<String, Arc<dyn EventsFactory>>,

    /// Supported Persistent Store backends.
    stores: HashMap<String, Arc<dyn StoreFactory>>,

    /// Supported Background Tasks backends.
    tasks: HashMap<String, Arc<dyn TasksFactory>>,
}

impl Backends {
    /// Lookup an [`EventsFactory`] by ID.
    pub fn events(&self, id: &str) -> Result<&dyn EventsFactory> {
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
        B: EventsFactory + 'static,
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

    /// Register a new factory for a Persistent Store implementation.
    ///
    /// # Panics
    ///
    /// This method panics if the identifier of the new Persistent Store backend is already in use.
    pub fn register_store<B, S>(&mut self, id: S, backend: B) -> &mut Self
    where
        B: StoreFactory + 'static,
        S: Into<String>,
    {
        match self.stores.entry(id.into()) {
            Entry::Occupied(entry) => {
                panic!(
                    "a StoreBackend with id '{}' is already registered",
                    entry.key()
                )
            }
            Entry::Vacant(entry) => entry.insert(Arc::new(backend)),
        };
        self
    }

    /// Register a new factory for a Background Tasks queue implementation.
    ///
    /// # Panics
    ///
    /// This method panics if the identifier of the new Background Tasks backend is already in use.
    pub fn register_tasks<B, S>(&mut self, id: S, backend: B) -> &mut Self
    where
        B: TasksFactory + 'static,
        S: Into<String>,
    {
        match self.tasks.entry(id.into()) {
            Entry::Occupied(entry) => {
                panic!(
                    "a TasksBackend with id '{}' is already registered",
                    entry.key()
                )
            }
            Entry::Vacant(entry) => entry.insert(Arc::new(backend)),
        };
        self
    }

    /// Lookup a [`StoreFactory`] by ID.
    pub fn store(&self, id: &str) -> Result<&dyn StoreFactory> {
        let factory = self
            .stores
            .get(id)
            .ok_or_else(|| BackendNotFound::store(id))?;
        Ok(factory.as_ref())
    }

    /// Lookup a [`TasksFactory`] by ID.
    pub fn tasks(&self, id: &str) -> Result<&dyn TasksFactory> {
        let factory = self
            .tasks
            .get(id)
            .ok_or_else(|| BackendNotFound::tasks(id))?;
        Ok(factory.as_ref())
    }
}
