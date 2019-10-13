use replicante_util_upkeep::Upkeep;

use super::Component;
use crate::interfaces::api::APIRoot;
use crate::interfaces::Interfaces;
use crate::Result;

mod cluster;
mod clusters;
mod constants;
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
        WebUI::mount_unstable_api(interfaces);
        WebUI {}
    }

    fn mount_unstable_api(interfaces: &mut Interfaces) {
        let primary = interfaces.stores.primary.clone();
        let view = interfaces.stores.view.clone();
        let mut router = interfaces.api.router_for(&APIRoot::UnstableWebUI);
        router.get(
            "/cluster/:cluster/agents",
            self::cluster::Agents::new(primary.clone()),
            "/cluster/:cluster/agents",
        );
        router.get(
            "/cluster/:cluster/discovery",
            self::cluster::Discovery::new(primary.clone()),
            "/cluster/:cluster/discovery",
        );
        router.get(
            "/cluster/:cluster/events",
            self::cluster::Events::new(view.clone()),
            "/cluster/:cluster/events",
        );
        router.get(
            "/cluster/:cluster/meta",
            self::cluster::Meta::new(primary.clone()),
            "/cluster/:cluster/meta",
        );
        router.get(
            "/cluster/:cluster/nodes",
            self::cluster::Nodes::new(primary.clone()),
            "/cluster/:cluster/nodes",
        );
        router.get(
            "/clusters/find",
            self::clusters::Find::new(primary.clone()),
            "/clusters/find",
        );
        router.get(
            "/clusters/find/:query",
            self::clusters::Find::new(primary.clone()),
            "/clusters/find/:query",
        );
        router.get(
            "/clusters/top",
            self::clusters::Top::new(primary),
            "/clusters/top",
        );
        router.get("/events", self::events::Events::new(view), "/events");
    }
}

impl Component for WebUI {
    /// Noop method for standard interface.
    fn run(&mut self, _: &mut Upkeep) -> Result<()> {
        Ok(())
    }
}
