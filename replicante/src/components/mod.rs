use std::time::Duration;
use slog::Logger;

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
            config.discovery.clone(), Duration::from_secs(config.timeouts.agents_api),
            logger, interfaces
        );
        let webui = WebUI::new(interfaces);
        Ok(Components {
            discovery,
            webui,
        })
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
