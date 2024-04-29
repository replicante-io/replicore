//! Dependency injection to enable easy access to Process Global resources.
use std::sync::RwLock;

use once_cell::sync::Lazy;

use replicore_auth::access::Authoriser;
use replicore_auth::identity::Authenticator;
use replicore_conf::Conf;
use replicore_context::Context;
use replicore_events::emit::Events;
use replicore_oaction::OActionRegistry;
use replicore_store::Store;
use replicore_tasks::submit::Tasks;

mod clients;

pub use self::clients::Clients;

/// Singleton instance of the Process Globals container.
static GLOBAL_INJECTOR: Lazy<RwLock<Option<Injector>>> = Lazy::new(|| RwLock::new(None));

/// Container for all process global dependencies to be injected in other components.
#[derive(Clone)]
pub struct Injector {
    /// Interface to determine the identity attached to requests being made.
    pub authenticator: Authenticator,

    /// Interface to verify permissions an entity has to perform an action on a resource.
    pub authoriser: Authoriser,

    /// API client factories for the control plane to interact with managed resources.
    pub clients: Clients,

    /// Process global configuration.
    pub conf: Conf,

    /// Process global context to derive scoped contexts from.
    pub context: Context,

    /// Interface to emit system events.
    pub events: Events,

    /// Registry of all orchestrator actions known to the process.
    pub oactions: OActionRegistry,

    /// Interface to persist state.
    pub store: Store,

    /// Interface to submit background tasks execution.
    pub tasks: Tasks,
}

impl Injector {
    /// Get the globally set [`Injector`] instance.
    ///
    /// # Panics
    ///
    /// Panics if no [`Injector`] was set during process initialisation.
    pub fn global() -> Injector {
        GLOBAL_INJECTOR
            .read()
            .expect("GLOBAL_INJECTOR RwLock poisoned")
            .as_ref()
            .expect("global injector is not initialised")
            .clone()
    }

    /// Set the [`Injector`] instance for the process to fetch with [`Injector::global`].
    ///
    /// # Panics
    ///
    /// Panics if an [`Injector`] has already been set.
    pub fn set_global(injector: Injector) {
        // Obtain a lock to initialise the global injector.
        let mut global_injector = GLOBAL_INJECTOR
            .write()
            .expect("GLOBAL_INJECTOR RwLock poisoned");

        // If the global injector is already initialised panic (without poisoning the lock).
        if global_injector.is_some() {
            drop(global_injector);
            panic!("global injector already initialised");
        }

        // Set the global injector for the process.
        slog::trace!(
            injector.context.logger,
            "Initialising Global Injector for the process"
        );
        *global_injector = Some(injector);
    }
}

#[cfg(any(test, feature = "test-fixture"))]
pub struct InjectorFixture {
    pub injector: Injector,
    pub events: replicore_events::emit::EventsFixture,
}

#[cfg(any(test, feature = "test-fixture"))]
impl Injector {
    /// [`Injector`] instance to be used with unit tests.
    pub fn fixture() -> InjectorFixture {
        let events = replicore_events::emit::EventsFixture::new();
        let authoriser = Authoriser::wrap(
            replicore_auth_insecure::Unrestricted,
            events.backend().into(),
        );
        let conf = Conf {
            events: replicore_conf::BackendConf {
                backend: "unittest".into(),
                options: Default::default(),
            },
            http: Default::default(),
            runtime: Default::default(),
            store: replicore_conf::BackendConf {
                backend: "unittest".into(),
                options: Default::default(),
            },
            tasks: replicore_conf::TasksConf {
                service: replicore_conf::BackendConf {
                    backend: "unittest".into(),
                    options: Default::default(),
                },
                executor: Default::default(),
            },
            telemetry: Default::default(),
        };
        let injector = Injector {
            authenticator: replicore_auth_insecure::Anonymous.into(),
            authoriser,
            clients: Clients::empty(),
            conf,
            context: Context::fixture(),
            events: events.backend().into(),
            oactions: OActionRegistry::build().finish(),
            store: Store::fixture(),
            tasks: Tasks::fixture().backend().into(),
        };
        InjectorFixture { injector, events }
    }
}
