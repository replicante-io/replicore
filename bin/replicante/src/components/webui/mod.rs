use replicante_util_upkeep::Upkeep;

use super::Component;
use crate::interfaces::Interfaces;
use crate::Result;

mod cluster;
mod clusters;
mod events;

/// Component to mount /webui endpoints.
///
/// These endpoints are designed to provide the Replicante WebUI node project
/// access to data without having direct access to the datastore.
///
/// This avoids having to coordinate datastore format between core and webui.
pub struct WebUI {}

impl WebUI {
    /// Create a new component and mounts all `/webui` endpoints.
    pub fn new(interfaces: &mut Interfaces) -> WebUI {
        self::cluster::Discovery::attach(interfaces);
        self::cluster::Meta::attach(interfaces);
        self::clusters::Find::attach(interfaces);
        self::clusters::Top::attach(interfaces);
        self::events::Events::attach(interfaces);
        WebUI {}
    }
}

impl Component for WebUI {
    /// Noop method for standard interface.
    fn run(&mut self, _: &mut Upkeep) -> Result<()> {
        Ok(())
    }
}
