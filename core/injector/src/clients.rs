//! Container for various client registries to be injected in other components.
use replicore_clients_platform::PlatformClients;

/// Container for various client registries to be injected in other components.
#[derive(Clone, Default)]
pub struct Clients {
    /// Configured factories for Platform API clients.
    pub platform: PlatformClients,
}

impl Clients {
    /// Initialise registries with no factories configured.
    pub fn empty() -> Clients {
        Clients {
            platform: PlatformClients::empty(),
        }
    }
}
