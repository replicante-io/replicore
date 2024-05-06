//! Collection of orchestrator action implementations known to the control plane.
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;

use crate::OActionHandler;

/// Default timeout for orchestrator actions.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(24 * 60 * 60);

/// Metadata attached to orchestrator action implementations.
#[derive(Debug)]
pub struct OActionMetadata {
    /// Identifier of the orchestrator action implementation.
    pub kind: String,

    /// Maximum time the action can stay running before it is failed and abandoned.
    pub timeout: Duration,

    /// [`OActionHandler`] to invoke when an `OAction` needs to run.
    pub handler: Box<dyn OActionHandler>,
}

impl OActionMetadata {
    /// Initialise an [`OActionMetadata`] builder.
    pub fn build<K, H>(kind: K, handler: H) -> OActionMetadataBuilder
    where
        K: Into<String>,
        H: OActionHandler + 'static,
    {
        OActionMetadataBuilder {
            kind: kind.into(),
            timeout: DEFAULT_TIMEOUT,
            handler: Box::new(handler),
        }
    }
}

impl IntoIterator for OActionMetadata {
    type Item = OActionMetadata;
    type IntoIter = std::array::IntoIter<Self::Item, 1>;

    fn into_iter(self) -> Self::IntoIter {
        [self].into_iter()
    }
}

/// Incrementally build an [`OActionMetadata`].
pub struct OActionMetadataBuilder {
    kind: String,
    timeout: Duration,
    handler: Box<dyn OActionHandler>,
}

impl OActionMetadataBuilder {
    /// Complete the build process.
    pub fn finish(self) -> OActionMetadata {
        OActionMetadata {
            kind: self.kind,
            timeout: self.timeout,
            handler: self.handler,
        }
    }

    /// Set the timeout after witch `OAction`s are failed and abandoned.
    pub fn timeout(&mut self, timeout: Duration) -> &mut Self {
        self.timeout = timeout;
        self
    }
}

/// Collection of [`OActionMetadata`] records known to the control plane.
#[derive(Clone, Debug)]
pub struct OActionRegistry {
    /// Map of orchestrator action ID to handling metadata.
    entries: Arc<HashMap<String, OActionMetadata>>,
}

impl OActionRegistry {
    /// Begin building an empty [`OActionRegistry`] instance.
    pub fn build() -> OActionRegistryBuilder {
        OActionRegistryBuilder::default()
    }

    /// Lookup the metadata for an action `kind`.
    pub fn lookup(&self, kind: &str) -> Result<&OActionMetadata> {
        self.entries
            .get(kind)
            .ok_or(crate::errors::OActionNotFound::from(kind))
            .map_err(anyhow::Error::from)
    }
}

/// Incrementally build [`OActionRegistry`]s.
#[derive(Debug, Default)]
pub struct OActionRegistryBuilder {
    entries: HashMap<String, OActionMetadata>,
}

impl OActionRegistryBuilder {
    /// Complete building the registry instance.
    pub fn finish(self) -> OActionRegistry {
        OActionRegistry {
            entries: Arc::new(self.entries),
        }
    }

    /// Register the metadata for a new orchestrator action.
    /// 
    /// # Panics
    /// 
    /// This method panics if the `kind` identifier is already registered.
    pub fn register(&mut self, metadata: OActionMetadata) -> &mut Self {
        if self.entries.contains_key(&metadata.kind) {
            panic!(
                "orchestrator action {} cannot be registered more then once",
                metadata.kind,
            );
        }

        let kind = metadata.kind.clone();
        self.entries.insert(kind, metadata);
        self
    }
}
