use replicante_util_upkeep::Upkeep;

use super::Component;
use crate::interfaces::Interfaces;
use crate::Result;

mod cluster;
mod clusters;
mod constants;
mod events;

/// Component to mount WebUI endpoints.
///
/// These endpoints are designed to provide the Replicante WebUI node project
/// access to data without having direct access to the datastore.
///
/// This avoids having to coordinate datastore format between core and webui.
pub struct WebUI {}

impl WebUI {
    pub fn new(interfaces: &mut Interfaces) -> WebUI {
        let cluster = self::cluster::configure(interfaces);
        let clusters = self::clusters::configure(interfaces);
        let events = self::events::configure(interfaces);
        interfaces.api.configure(cluster);
        interfaces.api.configure(clusters);
        interfaces.api.configure(events);
        WebUI {}
    }
}

impl Component for WebUI {
    /// Noop method for standard interface.
    fn run(&mut self, _: &mut Upkeep) -> Result<()> {
        Ok(())
    }
}
