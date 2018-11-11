/// Components enabling configuration.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct ComponentsConfig {
    /// Default status for all components that are not explicitly enabled/disabled.
    #[serde(default = "ComponentsConfig::default_default", rename = "_default")]
    default: bool,

    /// Enable agent discovery.
    discovery: Option<bool>,

    /// Enable Grafana Annotations API endpoints (optional).
    grafana: Option<bool>,

    /// Enable the WebUI API endpoints (optional).
    webui: Option<bool>,

    /// Enable the tasks workers pool component.
    workers: Option<bool>,
}

impl Default for ComponentsConfig {
    fn default() -> Self {
        Self {
            default: Self::default_default(),
            discovery: None,
            grafana: None,
            webui: None,
            workers: None,
        }
    }
}

impl ComponentsConfig {
    /// Default `_default` value used by serde.
    fn default_default() -> bool { true }
}

impl ComponentsConfig {
    /// Check if the discovery component is enabled.
    pub fn discovery(&self) -> bool {
        self.discovery.unwrap_or(self.default)
    }

    /// Check if the Grafana Annotations endpoints component is enabled.
    pub fn grafana(&self) -> bool {
        self.grafana.unwrap_or(self.default)
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
