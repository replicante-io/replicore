use prometheus::Registry;
use slog::Logger;

use replicante_data_aggregator::Aggregator;
use replicante_data_fetcher::Fetcher;

use super::Config;
use super::Interfaces;
use super::Result;
use super::metrics::COMPONENTS_ENABLED;


mod discovery;
mod grafana;
mod webui;
mod workers;

pub use self::discovery::Config as DiscoveryConfig;

use self::discovery::DiscoveryComponent as Discovery;
use self::grafana::Grafana;
use self::webui::WebUI;
use self::workers::Workers;


/// Helper macro to keep `Components::run` simpler in the presence of optional components.
macro_rules! component_run {
    ($component:expr) => {
        if let Some(component) = $component {
            component.run()?;
        }
    };
}

/// Helper macro to keep `Components::wait_all` simpler in the presence of optional components.
macro_rules! component_wait {
    ($component:expr) => {
        if let Some(component) = $component {
            component.wait()?;
        }
    };
}

/// Helper function to keep `Components::new` simpler in the presence of optional components.
fn component_new<C, F>(
    component: &str, mode: &str, enabled: bool, logger: &Logger, factory: F
) -> Option<C>
    where F: FnOnce() -> C,
{
    info!(
        logger, "Initialising component if enabled";
        "component" => component, "type" => mode, "enabled" => enabled
    );
    if enabled {
        COMPONENTS_ENABLED.with_label_values(&[component, mode]).set(1.0);
        Some(factory())
    } else {
        COMPONENTS_ENABLED.with_label_values(&[component, mode]).set(0.0);
        None
    }
}


/// A container for replicante components.
///
/// This container is useful to:
///
///   1. Have one argument passed arround for injection instead of many.
///   2. Store thread [`JoinHandle`]s to join on [`Drop`].
///
/// [`Drop`]: std/ops/trait.Drop.html
/// [`JoinHandle`]: std/thread/struct.JoinHandle.html
pub struct Components {
    discovery: Option<Discovery>,
    grafana: Option<Grafana>,
    webui: Option<WebUI>,
    workers: Option<Workers>,
}

impl Components {
    /// Creates and configures components.
    pub fn new(config: &Config, logger: Logger, interfaces: &mut Interfaces) -> Result<Components> {
        let discovery = component_new(
            "discovery", "required", config.components.discovery(), &logger, || Discovery::new(
                config.discovery.clone(), config.events.snapshots.clone(), logger.clone(), interfaces
            )
        );
        let grafana = component_new(
            "grafana", "optional", config.components.grafana(), &logger,
            || Grafana::new(interfaces)
        );
        let webui = component_new(
            "webui", "optional", config.components.webui(), &logger, || WebUI::new(interfaces)
        );
        let workers = component_new(
            "workers", "required", config.components.workers(), &logger,
            || Workers::new(interfaces, logger.clone(), config.clone())
        );
        let workers = match workers {
            Some(Err(error)) => return Err(error),
            Some(Ok(workers)) => Some(workers),
            None => None,
        };
        Ok(Components {
            discovery,
            grafana,
            webui,
            workers,
        })
    }

    /// Attemps to register all components metrics with the Registry.
    ///
    /// Metrics that fail to register are logged and ignored.
    pub fn register_metrics(logger: &Logger, registry: &Registry) {
        self::discovery::register_metrics(logger, registry);
        self::workers::register_metrics(logger, registry);
        ::replicante_agent_client::register_metrics(logger, registry);
        Aggregator::register_metrics(logger, registry);
        Fetcher::register_metrics(logger, registry);
    }


    /// Performs any final configuration and starts background threads.
    pub fn run(&mut self) -> Result<()> {
        component_run!(self.discovery.as_mut());
        component_run!(self.grafana.as_mut());
        component_run!(self.webui.as_mut());
        component_run!(self.workers.as_mut());
        Ok(())
    }

    /// Waits for all interfaces to terminate.
    pub fn wait_all(&mut self) -> Result<()> {
        component_wait!(self.discovery.as_mut());
        component_wait!(self.grafana.as_mut());
        component_wait!(self.webui.as_mut());
        component_wait!(self.workers.as_mut());
        Ok(())
    }
}
