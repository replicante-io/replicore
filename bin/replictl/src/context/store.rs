use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Context as _;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use slog::debug;
use tokio::fs::File;
use tokio::fs::OpenOptions;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::ErrorKind;

use super::Context;
use crate::errors::ContextNotFound;
use crate::utils::resolve_home;
use crate::Globals;

const DEFAULT_STORE_PATH: &str = "~/.config/replictl/contexts";

/// Store all known contexts, persisting them to disk.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ContextStore {
    /// Pointer to the currently active context, if any.
    #[serde(default, rename = "__active_context")]
    active: Option<String>,

    /// Collection of known contexts.
    #[serde(flatten)]
    contexts: BTreeMap<String, Context>,
}

impl ContextStore {
    /// Load the context store and return the active context, if it exists.
    pub async fn active(globals: &Globals) -> Result<Context> {
        let store = ContextStore::load(globals).await?;
        store.get_active(globals)
    }

    /// Determine the name of the active context.
    pub fn active_id<'a>(&'a self, globals: &'a Globals) -> &'a str {
        globals
            .cli
            .context
            .context
            .as_deref()
            .or(self.active.as_deref())
            .unwrap_or(super::DEFAULT_CONTEXT)
    }

    /// Find a context in the store, if present.
    pub fn get(&self, id: &str) -> Option<Context> {
        self.contexts.get(id).cloned()
    }

    /// find the active context in the store, if one matching the active name exists.
    pub fn get_active(&self, globals: &Globals) -> Result<Context> {
        let name = self.active_id(globals);
        self.get(name)
            .ok_or_else(|| anyhow::anyhow!(ContextNotFound::for_name(name)))
    }

    /// Load the contexts store from disk.
    // NOTE: the &Cli args is to add ContextStore options (location) in the future.
    pub async fn load(globals: &Globals) -> Result<ContextStore> {
        // Async load the store file into a buffer.
        let path = resolve_home(DEFAULT_STORE_PATH)?;
        debug!(globals.logger, "Loading contexts store from disk"; "path" => &path);
        let mut reader = match File::open(&path).await {
            Ok(file) => file,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(ContextStore::empty()),
            Err(error) => {
                return Err(error).context(format!("unable to open contexts store from {}", &path))
            }
        };
        let mut buffer = Vec::new();
        reader
            .read_to_end(&mut buffer)
            .await
            .with_context(|| format!("unable to read contexts store from {}", &path))?;

        // Decode the store from the buffer and return it.
        let store = serde_yaml::from_slice(&buffer)
            .with_context(|| format!("unable to YAML decode contexts store from {}", path))?;
        Ok(store)
    }

    /// Iterate over contexts in the store.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Context)> {
        self.contexts
            .iter()
            .map(|(name, context)| (name.as_str(), context))
    }

    /// Remove the named context from the store, clearing the active context if needed.
    pub fn remove(&mut self, name: &str) {
        if self.active.as_ref().map(|n| n == name).unwrap_or(false) {
            self.active = None;
        }
        self.contexts.remove(name);
    }

    /// Write the context store to disk.
    ///
    /// If the path containing the credentials file does not exist it will be created.
    // NOTE: the &Cli args is to add ContextStore options (location) in the future.
    pub async fn save(&self, globals: &Globals) -> Result<()> {
        let path = resolve_home(DEFAULT_STORE_PATH)?;
        debug!(globals.logger, "Persisting contexts store to disk"; "path" => &path);
        ensure_store_path(globals, &path).await?;

        // Encode the store to a buffer so it can be written to disk asynchronously.
        let mut buffer = Vec::new();
        serde_yaml::to_writer(&mut buffer, self)
            .with_context(|| format!("unable to YAML encode contexts store to {}", &path))?;
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&path)
            .await
            .with_context(|| format!("unable to open contexts store at {}", &path))?;
        file.write_all(&buffer)
            .await
            .with_context(|| format!("unable to write contexts store to {}", &path))?;
        file.flush()
            .await
            .with_context(|| format!("unable to flush contexts store to {}", path))
    }

    /// Set the active context name persisted by the store.
    pub fn set_active_id(&mut self, name: Option<String>) {
        self.active = name;
    }

    /// Insert or update a context.
    pub fn upsert<S>(&mut self, name: S, context: Context)
    where
        S: Into<String>,
    {
        self.contexts.insert(name.into(), context);
    }
}

impl ContextStore {
    /// Return an empty ContextStore.
    fn empty() -> ContextStore {
        ContextStore {
            active: None,
            contexts: BTreeMap::default(),
        }
    }
}

// Create the contexts store store parent directory if needed.
async fn ensure_store_path(globals: &Globals, path: &str) -> Result<()> {
    let parent = Path::new(path);
    let parent = match parent.parent() {
        None => return Ok(()),
        Some(parent) => parent,
    };
    if parent.exists() {
        return Ok(());
    }
    debug!(globals.logger, "Creating parent directories for contexts store file"; "path" => path);
    tokio::fs::create_dir_all(parent)
        .await
        .with_context(|| "unable to create parent directories for the contexts store file")
}
