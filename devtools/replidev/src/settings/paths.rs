use crate::conf::Project;

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
        let configs = format!("./devtools/deps/configs/{}", pod_name);
        let data = format!("./devtools/deps/data/{}", pod_name);
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

pub struct PlayPod {
    configs: String,
    data: String,
}

impl PlayPod {
    pub fn new(store: &str, cluster_id: &str, node: &str) -> Self {
        let configs = format!("./stores/{}", store);
        let data = format!("./data/nodes/{}/{}", cluster_id, node);
        PlayPod { configs, data }
    }
}

impl Paths for PlayPod {
    fn configs(&self) -> &str {
        &self.configs
    }

    fn data(&self) -> &str {
        &self.data
    }
}
