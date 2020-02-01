use prometheus::Registry;
use slog::info;
use slog::Logger;

use replicante_util_upkeep::Upkeep;

use super::metrics::COMPONENTS_ENABLED;
use super::Config;
use super::Interfaces;
use super::Result;

mod core_api;
mod discovery;
mod events_indexer;
mod grafana;
mod update_checker;
mod webui;
mod workers;

pub use self::discovery::Config as DiscoveryConfig;

use self::core_api::CoreAPI;
use self::discovery::DiscoveryComponent as Discovery;
use self::events_indexer::EventsIndexer;
use self::grafana::Grafana;
use self::update_checker::UpdateChecker;
use self::webui::WebUI;
use self::workers::Workers;

/// Helper function to keep `Components::new` simpler in the presence of optional components.
fn init_component<C, F>(
    components: &mut Vec<Box<dyn Component>>,
    component: &str,
    mode: &str,
    enabled: bool,
    logger: &Logger,
    factory: F,
) where
    C: Component + 'static,
    F: FnOnce() -> C,
{
    init_component_result(components, component, mode, enabled, logger, || {
        Ok(factory())
    })
    .expect("component failed. Use init_component_result to propagate");
}

/// Helper function to keep `Components::new` simpler in the presence of optional components.
fn init_component_result<C, F>(
    components: &mut Vec<Box<dyn Component>>,
    component: &str,
    mode: &str,
    enabled: bool,
    logger: &Logger,
    factory: F,
) -> Result<()>
where
    C: Component + 'static,
    F: FnOnce() -> Result<C>,
{
    info!(
        logger,
        "Initialising component if enabled";
        "component" => component,
        "type" => mode,
        "enabled" => enabled,
    );
    if enabled {
        COMPONENTS_ENABLED
            .with_label_values(&[component, mode])
            .set(1.0);
        components.push(Box::new(factory()?));
    } else {
        COMPONENTS_ENABLED
            .with_label_values(&[component, mode])
            .set(0.0);
    }
    Ok(())
}

/// Generic replicante core component that does something.
trait Component {
    /// Start the component, registering any thread or shutdown signals.
    fn run(&mut self, upkeep: &mut Upkeep) -> Result<()>;
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
    components: Vec<Box<dyn Component>>,
}

impl Components {
    /// Creates and configures components.
    pub fn new(config: &Config, logger: Logger, interfaces: &mut Interfaces) -> Result<Components> {
        let mut components = Vec::new();
        init_component(
            &mut components,
            "core_api",
            "required",
            config.components.core_api(),
            &logger,
            || CoreAPI::new(logger.clone(), interfaces),
        );
        init_component(
            &mut components,
            "discovery",
            "required",
            config.components.discovery(),
            &logger,
            || {
                Discovery::new(
                    config.discovery.clone(),
                    config.events.snapshots.clone(),
                    logger.clone(),
                    interfaces,
                )
            },
        );
        init_component(
            &mut components,
            "events_indexer",
            "required",
            config.components.events_indexer(),
            &logger,
            || EventsIndexer::new(logger.clone(), interfaces),
        );
        init_component(
            &mut components,
            "grafana",
            "optional",
            config.components.grafana(),
            &logger,
            || Grafana::new(interfaces),
        );
        init_component(
            &mut components,
            "update_checker",
            "optional",
            config.components.update_checker(),
            &logger,
            || UpdateChecker::new(logger.clone()),
        );
        init_component(
            &mut components,
            "webui",
            "optional",
            config.components.webui(),
            &logger,
            || WebUI::new(interfaces),
        );
        init_component_result(
            &mut components,
            "workers",
            "required",
            config.components.workers(),
            &logger,
            || Workers::new(interfaces, logger.clone(), config.clone()),
        )?;
        Ok(Components { components })
    }

    /// Attemps to register all components metrics with the Registry.
    ///
    /// Metrics that fail to register are logged and ignored.
    pub fn register_metrics(logger: &Logger, registry: &Registry) {
        self::core_api::register_metrics(logger, registry);
        self::discovery::register_metrics(logger, registry);
        self::workers::register_metrics(logger, registry);
    }

    /// Performs any final configuration and starts background threads.
    pub fn run(&mut self, upkeep: &mut Upkeep) -> Result<()> {
        for component in &mut self.components {
            component.run(upkeep)?;
        }
        Ok(())
    }
}
