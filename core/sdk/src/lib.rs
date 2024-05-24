//! Reusable Control Plane business logic in the form of an SDK.
use replicore_injector::Injector;

mod oaction;

pub mod constants;
pub mod errors;

/// Reusable Control Plane business logic in the form of an SDK.
#[derive(Clone)]
pub struct CoreSDK {
    injector: Injector,
}

impl CoreSDK {
    /// Initialise a new [`CoreSDK`] from the provided injector.
    #[deprecated(note = "use From<Injector>")]
    pub fn from_injector(injector: Injector) -> Self {
        Self { injector }
    }

    /// Initialise a new [`CoreSDK`] from the globally initialised injector.
    pub fn from_globals() -> Self {
        let injector = Injector::global();
        Self::from(injector)
    }
}

impl From<&Injector> for CoreSDK {
    fn from(injector: &Injector) -> Self {
        let injector = injector.clone();
        Self { injector }
    }
}

impl From<Injector> for CoreSDK {
    fn from(injector: Injector) -> Self {
        Self { injector }
    }
}
