use failure::ResultExt;
use prometheus::Registry;
use slog::info;
use slog::Logger;

use replicante_util_upkeep::Upkeep;

use replicore_component_discovery_scheduler::Config as DiscoveryConfig;
use replicore_component_orchestrator_scheduler::Config as OrchestratorConfig;

use super::metrics::COMPONENTS_ENABLED;
use super::Config;
use super::Error;
use super::ErrorKind;
use super::Interfaces;
use super::Result;

mod core_api;
mod grafana;
mod update_checker;
mod webui;
mod workers;

use self::core_api::CoreAPI;
use self::grafana::Grafana;
use self::update_checker::UpdateChecker;
use self::webui::WebUI;
use self::workers::Workers;

/// Wrap a component defined ouside of this crate with the `Component` trait.
///
/// NOTE: this is a temporary solution to address two shortcomings:
///   * Components should be defined externally but were not initially (they will be moved).
///   * The codebase will go async in the future so interfaces will be redesigned then.
macro_rules! impl_component {
    ($name:ident, $real:ty) => {
        struct $name($real);

        impl Component for $name {
            fn run(&mut self, upkeep: &mut Upkeep) -> Result<()> {
                self.0
                    .run(upkeep)
                    .with_context(|_| ErrorKind::ComponentFailed(stringify!($name)))
                    .map_err(Error::from)
            }
        }
    };
}

impl_component!(
    Discovery,
    replicore_component_discovery_scheduler::Discovery
);
impl Discovery {
    fn new(config: DiscoveryConfig, interfaces: &Interfaces) -> Discovery {
        let logger = interfaces.logger.clone();
        let store = interfaces.stores.primary.clone();
        let tasks = interfaces.tasks.clone();
        let tracer = interfaces.tracing.tracer();
        let component = replicore_component_discovery_scheduler::Discovery::new(
            interfaces.coordinator.clone(),
            config,
            logger,
            store,
            tasks,
            tracer,
        );
        Discovery(component)
    }
}

impl_component!(
    OrchestratorScheduler,
    replicore_component_orchestrator_scheduler::OrchestratorScheduler
);
impl OrchestratorScheduler {
    fn new(config: OrchestratorConfig, interfaces: &Interfaces) -> OrchestratorScheduler {
        let component = replicore_component_orchestrator_scheduler::OrchestratorScheduler::new(
            interfaces.coordinator.clone(),
            config,
            interfaces.logger.clone(),
        );
        OrchestratorScheduler(component)
    }
}

impl_component!(ViewUpdater, replicore_component_viewupdater::ViewUpdater);
impl ViewUpdater {
    fn new(interfaces: &Interfaces) -> ViewUpdater {
        let events = interfaces.streams.events.clone();
        let logger = interfaces.logger.clone();
        let store = interfaces.stores.view.clone();
        let tracer = interfaces.tracing.tracer();
        let component =
            replicore_component_viewupdater::ViewUpdater::new(events, logger, store, tracer);
        ViewUpdater(component)
    }
}

/// Helper function to keep `Components::new` simpler in the presence of optional components.
macro_rules! init_components {
    {
        let logger = $logger:expr;
        $( component($name:literal, $mode:literal) {
            let enabled = $enabled:expr;
            $factory:expr
        })+
    } => {
        {
            let mut components: Vec<Box<dyn Component>> = Vec::new();
            let logger = $logger;
            $(
                let enabled = $enabled;
                if enabled {
                    info!(
                        logger,
                        "Initialising component";
                        "component" => $name,
                        "type" => $mode,
                        "enabled" => enabled,
                    );
                    COMPONENTS_ENABLED
                        .with_label_values(&[$name, $mode])
                        .set(1.0);
                    components.push(Box::new($factory));
                } else {
                    info!(
                        logger,
                        "Skipped component";
                        "component" => $name,
                        "type" => $mode,
                        "enabled" => enabled,
                    );
                    COMPONENTS_ENABLED
                        .with_label_values(&[$name, $mode])
                        .set(0.0);
                }
            )+
            components
        }
    };
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
        let components = init_components! {
            let logger = &logger;
            component("core_api", "required") {
                let enabled = config.components.core_api();
                CoreAPI::new(logger.clone(), interfaces)
            }
            component("discovery", "required") {
                let enabled = config.components.discovery();
                Discovery::new(
                    config.discovery.clone(),
                    interfaces,
                )
            }
            component("grafana", "optional") {
                let enabled = config.components.grafana();
                Grafana::new(interfaces)
            }
            component("orchestrator", "required") {
                let enabled = config.components.orchestrator();
                OrchestratorScheduler::new(
                    config.orchestrator.clone(),
                    interfaces,
                )
            }
            component("update_checker", "optional") {
                let enabled = config.components.update_checker();
                UpdateChecker::new(logger.clone())
            }
            component("viewupdater", "required") {
                let enabled = config.components.viewupdater();
                ViewUpdater::new(interfaces)
            }
            component("webui", "optional") {
                let enabled = config.components.webui();
                WebUI::new(interfaces)
            }
            component("workers", "required") {
                let enabled = config.components.workers();
                Workers::new(interfaces, logger.clone(), config.clone())?
            }
        };
        Ok(Components { components })
    }

    /// Attemps to register all components metrics with the Registry.
    ///
    /// Metrics that fail to register are logged and ignored.
    pub fn register_metrics(logger: &Logger, registry: &Registry) {
        self::core_api::register_metrics(logger, registry);
        replicore_component_discovery_scheduler::register_metrics(logger, registry);
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
