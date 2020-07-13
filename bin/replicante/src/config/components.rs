use serde_derive::Deserialize;
use serde_derive::Serialize;

/// Components enabling configuration.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct ComponentsConfig {
    /// Default status for all components that are not explicitly enabled/disabled.
    #[serde(default = "ComponentsConfig::default_default", rename = "_default")]
    default: bool,

    /// Enable replicante core API endpoints.
    #[serde(default)]
    core_api: Option<bool>,

    /// Enable agent discovery.
    #[serde(default)]
    discovery: Option<bool>,

    /// Enable Grafana Annotations API endpoints (optional).
    #[serde(default)]
    grafana: Option<bool>,

    /// Enable the update checker (optional).
    #[serde(default = "ComponentsConfig::default_false")]
    update_checker: bool,

    /// Enable the view DB updater.
    viewupdater: Option<bool>,

    /// Enable the WebUI API endpoints (optional).
    #[serde(default)]
    webui: Option<bool>,

    /// Enable the tasks workers pool component.
    #[serde(default)]
    workers: Option<bool>,
}

impl Default for ComponentsConfig {
    fn default() -> Self {
        Self {
            default: Self::default_default(),
            core_api: None,
            discovery: None,
            grafana: None,
            update_checker: Self::default_false(),
            viewupdater: None,
            webui: None,
            workers: None,
        }
    }
}

impl ComponentsConfig {
    fn default_default() -> bool {
        true
    }

    fn default_false() -> bool {
        false
    }
}

impl ComponentsConfig {
    /// Check if the core API component is enabled.
    pub fn core_api(&self) -> bool {
        self.core_api.unwrap_or(self.default)
    }

    /// Check if the discovery component is enabled.
    pub fn discovery(&self) -> bool {
        self.discovery.unwrap_or(self.default)
    }

    /// Check if the Grafana Annotations endpoints component is enabled.
    pub fn grafana(&self) -> bool {
        self.grafana.unwrap_or(self.default)
    }

    /// Check if the update checker component is enabled.
    pub fn update_checker(&self) -> bool {
        self.update_checker
    }

    /// Check if the view DB updater component is enabled.
    pub fn viewupdater(&self) -> bool {
        self.viewupdater.unwrap_or(self.default)
    }

    /// Check if the WebUI endpoints component is enabled.
    pub fn webui(&self) -> bool {
        self.webui.unwrap_or(self.default)
    }

    /// Check if the Workers component is enabled.
    pub fn workers(&self) -> bool {
        self.workers.unwrap_or(self.default)
    }
}
