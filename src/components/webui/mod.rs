use super::super::Result;
use super::super::interfaces::Interfaces;


mod cluster;
mod clusters;


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
        self::cluster::Meta::attach(interfaces);
        self::clusters::Find::attach(interfaces);
        self::clusters::Top::attach(interfaces);
        WebUI {}
    }

    /// Noop method for standard interface.
    pub fn run(&self) -> Result<()> {
        Ok(())
    }

    /// Noop method for standard interface.
    pub fn wait(&self) -> Result<()> {
        Ok(())
    }
}
