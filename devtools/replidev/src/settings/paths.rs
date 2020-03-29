use crate::conf::Project;

const DEPS_BASE: &str = "./devtools/deps";

/// Pod related paths factory.
///
/// Allows paths to be injected into pod definitions and tool invokations
/// based on the current project, pod or other factor.
pub trait Paths {
    /// Path to a pod's static, git committed, configuration files.
    fn configs(&self) -> &str;

    /// Path to a pod's persistent, git ignored, data files.
    fn data(&self) -> &str;
}

impl dyn Paths {
    /// Path to the PKI store for the given project.
    pub fn pki(project: &Project) -> &'static str {
        match project {
            Project::Agents => "./devtools/pki",
            Project::Core => "./devtools/deps/pki",
            Project::Playground => "./data/pki",
        }
    }
}

/// Paths for pods in the `replidev deps` commands.
pub struct DepsPod {
    configs: String,
    data: String,
}

impl DepsPod {
    pub fn new(pod_name: &str) -> Self {
        let configs = format!("{}/configs/{}", DEPS_BASE, pod_name);
        let data = format!("{}/data/{}", DEPS_BASE, pod_name);
        DepsPod { configs, data }
    }
}

impl Paths for DepsPod {
    fn configs(&self) -> &str {
        &self.configs
    }

    fn data(&self) -> &str {
        &self.data
    }
}
