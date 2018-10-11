use std::time::Duration;
use prometheus::Registry;
use slog::Logger;

use replicante_agent_client::HttpClient as AgentHttpClient;
use replicante_data_aggregator::Aggregator;
use replicante_data_fetcher::Fetcher;

use super::Config;
use super::Interfaces;
use super::Result;


pub mod discovery;
pub mod webui;

use self::discovery::DiscoveryComponent as Discovery;
use self::webui::WebUI;


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
    discovery: Discovery,
    webui: WebUI,
}

impl Components {
    /// Creates and configures components.
    pub fn new(config: &Config, logger: Logger, interfaces: &mut Interfaces) -> Result<Components> {
        let discovery = Discovery::new(
            config.discovery.clone(), config.events.snapshots.clone(),
            Duration::from_secs(config.timeouts.agents_api), logger, interfaces
        );
        let webui = WebUI::new(interfaces);
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
        self.discovery.run()?;
        self.webui.run()?;
        Ok(())
    }

    /// Waits for all interfaces to terminate.
    pub fn wait_all(&mut self) -> Result<()> {
        self.discovery.wait()?;
        self.webui.wait()?;
        Ok(())
    }
}
