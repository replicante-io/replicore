use replicante_util_upkeep::Upkeep;

use super::super::interfaces::Interfaces;
use super::super::Result;

mod annotations;
mod check;

/// Component to mount /api/v1/grafana endpoints.
///
/// These endpoints are designed to provide an Annotations backend for the Grafana
/// [SimpleJson](https://grafana.com/plugins/grafana-simple-json-datasource) plugin.
pub struct Grafana {}

impl Grafana {
    /// Instantiate the Grafana component and mounts all `/api/v1/grafana` endpoints.
    pub fn new(interfaces: &mut Interfaces) -> Grafana {
        self::annotations::Annotations::attach(interfaces);
        self::check::Check::attach(interfaces);
        Grafana {}
    }

    /// Noop method for standard interface.
    pub fn run(&self, _: &mut Upkeep) -> Result<()> {
        Ok(())
    }
}
