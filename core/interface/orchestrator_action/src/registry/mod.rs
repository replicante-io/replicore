// ##############################################################################################
// *** ARCHITECTURE DESCRIPTION ***
// This note block describes why and how the OrchestratorActionRegistry can be accessed globally.
//
// # Why?
//
// To keep the Replicante Core code clean and flexible we want to make use of "initialise once"
// global objects in the style of `log` global logger.
//
// With this in place, any location that wishes to lookup `OrchestratorAction` can do so
// without having to constantly pass around and track registry objects and references.
//
// # How?
//
// Thanks to the "initialise once in main" use case for this global the code and implement
// pans out fairly easy and efficient:
//
// - The current global registry is store into a static `RwLock`, initially empty (stores `None`).
// - On initialisation, a registry is stored in the `RwLock` (stores `Some(...)`).
// - On initialisation, the system panics if already initialised.
// - For efficiency, access to the current global registry is managed by thread locals.
// - On first access within a thread:
//   - The `RwLock` is obtained for reading.
//   - A reference to the stored registry is copied to the thread local by the local initialiser.
//   - All future access from within the same thread can skip any locking.
// - If the current global registry is not initialised when a thread attempts access it will panic.
// - NOTE: this approach works because the registry CANNOT be changed once set.
//         If that was not the case then each thread would get different copy and drift over time!
//
// # Testing support
//
// Global state makes tests harder to write and potentially chaotic if they start clashing.
// For that reason this crate offers a `test-api` feature.
// When this feature is enabled the implementation of the global registry is changed so:
//
// - The current global registry static `RwLock` does NOT EVEN exist.
// - Threads will have independent copies of registry as fixtures.
//   - This is done with `thread_local` storing `RwLock<Option<...>>` directly.
// - Setting the current global registry will only set the registry for the current thread.
// - An additional API is available to clear the current thread's registry.
// - A RAII guard is available to call this reset API automatically as the test ends.
// - Duplicate initialisation of the same thread will still result in a panic.
// - Accessing the current registry is done over the same API.
// - Accessing an un-initialised registry still results in a panic.
// - A procedural macro is available to make test cleaner and quicker to write.
//
// Because testing support is essentially a completely different API it has its own dedicated
// tests implemented in `src/registry/test_api_tests.rs` which are run with
// `cargo test --features test-api`.
// ##############################################################################################
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::RwLock;

use anyhow::Result;

use crate::ActionAlreadyRegistered;
use crate::OrchestratorAction;
use crate::OrchestratorActionMetadata;

mod macros;

// When test are enabled but the `test-api` is off.
#[cfg(all(test, not(feature = "test-api")))]
mod tests;

// When test are enabled and the `test-api` is on.
#[cfg(all(test, feature = "test-api"))]
mod test_api_tests;

// Global registry is only defined outside of `test-api` mode.
#[cfg(not(feature = "test-api"))]
lazy_static::lazy_static! {
    /// Global OrchestratorActionRegistry instance currently active.
    ///
    /// Can only be set one for the whole process and is accessed through a more efficient
    /// thread-local read-only "cache".
    static ref ORCH_ACT_REG: RwLock<Option<Arc<OrchestratorActionRegistry>>> = RwLock::new(None);
}

// Thread local definitions when outside of `test-api` mode.
#[cfg(not(feature = "test-api"))]
thread_local! {
    /// Thread-local read-only "cache" for the current `OrchestratorActionRegistry`.
    static ORCH_ACT_REG_TLS: Arc<OrchestratorActionRegistry> = {
        let current = ORCH_ACT_REG
            .read()
            .expect("OrchestratorActionRegistry global state is poisoned");
        match current.as_ref() {
            Some(current) => Arc::clone(current),
            None => {
                // Drop the lock before panicking to avoid poisoning it for others.
                drop(current);
                panic!("accessed current OrchestratorActionRegistry before it is initialised");
            }
        }
    };
}

// Thread local definitions when in `test-api` mode.
#[cfg(feature = "test-api")]
thread_local! {
    /// When `test-api` is enabled then threads get isolated registries.
    static ORCH_ACT_REG_TLS: RwLock<Option<Arc<OrchestratorActionRegistry>>> = RwLock::new(None);
}

/// Registry of available orchestrator action implementations.
///
/// ## Global Registry
///
/// Registry instances are pretty simple and offer lookup logic.
///
/// To make code using the registry cleaner to write and to keep the overall architecture a bit
/// cleaner a "global registry" should be initialised once all actions are registered with the
/// builder.
///
/// * Access the global registry with `OrchestratorActionRegistry::current()`.
/// * Attempts to access the global registry before one is set result in a panic.
/// * Attempts to change the global registry once it is set result in a panic.
///
/// ### Testing support
///
/// To support testing code that makes use of the global registry an optional `test-api` feature
/// is provided by this crate. When enabled:
///
/// * Initialising the global registry with the standard method will panic.
///   This ensure tests always run with a fixture/localised registry.
/// * An additional API to configure a per-thread test fixture is available.
///
/// ## Example
///
/// ```ignore
/// // Add action implementations to the register.
/// let mut builder = OrchestratorActionRegistryBuilder::empty();
/// builder
///     .register("action.scope/id", ActionImplementation::registry_entry())
///     .expect("expect action to be registered");
///
/// // Build the registry and set it as the current registry.
/// builder.build_as_current();
///
/// // Lookup actions from the registry.
/// let action = OrchestratorActionRegistry::current().lookup("action.scope/id");
///
/// // And for tests:
/// #[test]
/// fn some_test_that_uses_the_registry() {
///   // Build the registry (ideally with fixtures).
///   let mut builder = ...
///   builder.build_as_current();
///
///   // Set up a guard to clear the registry when the test ends (even on fail).
///   let _ = TestRegistryClearGuard::default();
///
///   // Directly or indirectly access the registry as normal.
///   let register = OrchestratorActionRegistry::current();
/// }
/// ```
pub struct OrchestratorActionRegistry {
    actions: BTreeMap<String, OrchestratorActionRegistryEntry>,
}

impl OrchestratorActionRegistry {
    /// Access the current global `OrchestratorActionRegistry` instance.
    ///
    /// # Panic
    ///
    /// This method panics if called before the current global `OrchestratorActionRegistry`
    /// is initialised with `OrchestratorActionRegistryBuilder::build_as_current`.
    pub fn current() -> Arc<OrchestratorActionRegistry> {
        // Simply read the cache outside of test mode.
        #[cfg(not(feature = "test-api"))]
        return ORCH_ACT_REG_TLS.with(Arc::clone);

        // In test mode check if registry is set first.
        #[cfg(feature = "test-api")]
        ORCH_ACT_REG_TLS.with(|state| {
            let registry = state
                .read()
                .expect("OrchestratorActionRegistry test state is poisoned")
                .clone();
            registry.expect("accessed test OrchestratorActionRegistry before it is initialised")
        })
    }

    /// Iterate over registered action (&ID, &Handler) tuples.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &OrchestratorActionRegistryEntry)> {
        self.actions
            .iter()
            .map(|(id, action)| (id.as_str(), action))
    }

    /// Lookup an `OrchestratorAction` implementation from the registry.
    pub fn lookup(&self, id: &str) -> Option<&OrchestratorActionRegistryEntry> {
        self.actions.get(id)
    }
}

/// Builds a new OrchestratorActionRegistry instance.
#[derive(Default)]
pub struct OrchestratorActionRegistryBuilder {
    actions: BTreeMap<String, OrchestratorActionRegistryEntry>,
}

impl OrchestratorActionRegistryBuilder {
    /// Consume the builder to finish building the registry.
    pub fn build(self) -> OrchestratorActionRegistry {
        OrchestratorActionRegistry {
            actions: self.actions,
        }
    }

    /// Consume the builder to finish building the registry and sets it as global current.
    ///
    /// # Panic
    ///
    /// This method panics if the global registry is already initialised when this method is called.
    ///
    /// # Test mode
    ///
    /// When the `test-api` feature is enabled this method sets a per-thread registry.
    pub fn build_as_current(self) {
        let registry = Arc::new(self.build());

        // Set the global registry outside of test mode.
        #[cfg(not(feature = "test-api"))]
        {
            let mut current = ORCH_ACT_REG
                .write()
                .expect("OrchestratorActionRegistry global state is poisoned");
            if current.is_some() {
                // Make sure the lock is dropped BEFORE panic or it will poison the lock for all!!!
                drop(current);
                panic!("attempted dual initialisation of the current OrchestratorActionRegistry");
            }
            *current = Some(registry);
        }

        // In test mode set the local thread state only.
        #[cfg(feature = "test-api")]
        ORCH_ACT_REG_TLS.with(|state| {
            let mut current = state
                .write()
                .expect("OrchestratorActionRegistry test state is poisoned");
            if current.is_some() {
                // Make sure the lock is dropped BEFORE panic or it will poison the lock for all!!!
                drop(current);
                panic!("attempted dual initialisation of the current OrchestratorActionRegistry");
            }
            *current = Some(registry);
        });
    }

    /// Test API method to clear the currently set registry.
    #[cfg(feature = "test-api")]
    pub fn clear_current() {
        ORCH_ACT_REG_TLS.with(|state| {
            let mut current = state
                .write()
                .expect("OrchestratorActionRegistry test state is poisoned");
            *current = None;
        });
    }

    /// Start building and empty registry.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Register an `OrchestratorAction` implementation and metadata.
    pub fn register<S>(
        &mut self,
        id: S,
        entry: OrchestratorActionRegistryEntry,
    ) -> Result<&mut Self>
    where
        S: Into<String>,
    {
        let id = id.into();
        match self.actions.entry(id) {
            Entry::Occupied(found_entry) => {
                let id = found_entry.key().to_owned();
                anyhow::bail!(ActionAlreadyRegistered { id });
            }
            Entry::Vacant(found_entry) => {
                found_entry.insert(entry);
                Ok(self)
            }
        }
    }
}

/// A single entry in the orchestrator actions registry.
pub struct OrchestratorActionRegistryEntry {
    /// Implementation of the `OrchestratorAction` trait to execute the action.
    pub handler: Box<dyn OrchestratorAction>,

    /// Metadata attached to the `OrchestratorAction`.
    pub metadata: OrchestratorActionMetadata,
}

/// Automatically clear the current test registry on drop.
///
/// This struct is helpful when writing tests to ensure the registry is unset at the end.
#[cfg(feature = "test-api")]
#[derive(Default)]
pub struct TestRegistryClearGuard {}

#[cfg(feature = "test-api")]
impl Drop for TestRegistryClearGuard {
    fn drop(&mut self) {
        // This implementation of clearing ignores poisoned locks to avoid double panics.
        ORCH_ACT_REG_TLS.with(|state| {
            if let Ok(mut current) = state.write() {
                *current = None;
            }
        });
    }
}
