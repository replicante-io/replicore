use std::time::Duration;
use prometheus::Registry;
use slog::Logger;

use replicante_agent_client::HttpClient as AgentHttpClient;
use replicante_data_aggregator::Aggregator;
use replicante_data_fetcher::Fetcher;

use super::Config;
use super::Interfaces;
use super::Result;
use super::metrics::COMPONENTS_ENABLED;


pub mod discovery;
pub mod webui;

use self::discovery::DiscoveryComponent as Discovery;
use self::webui::WebUI;


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
    component: &str, mode: &str, enabled: bool, logger: Logger, factory: F
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
    webui: Option<WebUI>,
}

impl Components {
    /// Creates and configures components.
    pub fn new(config: &Config, logger: Logger, interfaces: &mut Interfaces) -> Result<Components> {
        let discovery = component_new(
            "discovery", "required", config.components.discovery(), logger.clone(), || {
                Discovery::new(
                    config.discovery.clone(), config.events.snapshots.clone(),
                    Duration::from_secs(config.timeouts.agents_api), logger.clone(), interfaces
                )
            }
        );
        let webui = component_new(
            "webui", "optional", config.components.webui(), logger.clone(),
            || WebUI::new(interfaces)
        );
        Ok(Components {
            discovery,
            webui,
        })
    }

    /// Attemps to register all components metrics with the Registry.
    ///
    /// Metrics that fail to register are logged and ignored.
    pub fn register_metrics(logger: &Logger, registry: &Registry) {
        self::discovery::register_metrics(logger, registry);
        AgentHttpClient::register_metrics(logger, registry);
        Aggregator::register_metrics(logger, registry);
        Fetcher::register_metrics(logger, registry);
    }


    /// Performs any final configuration and starts background threads.
    pub fn run(&mut self) -> Result<()> {
        component_run!(self.discovery.as_mut());
        component_run!(self.webui.as_mut());
        Ok(())
    }

    /// Waits for all interfaces to terminate.
    pub fn wait_all(&mut self) -> Result<()> {
        component_wait!(self.discovery.as_mut());
        component_wait!(self.webui.as_mut());
        Ok(())
    }
}
