use std::collections::BTreeMap;

use serde::Deserialize;

mod exec_cmd;
mod pod_ps_cmd;
mod pod_start_cmd;
mod pod_stop_cmd;
mod unshare_cmd;
mod variables;

pub use self::exec_cmd::exec;
pub use self::pod_ps_cmd::pod_ps;
pub use self::pod_start_cmd::pod_start;
pub use self::pod_stop_cmd::pod_stop;
pub use self::unshare_cmd::unshare;
pub use self::variables::Variables;

/// Definition of a pod to start with podman.
#[derive(Debug, Deserialize)]
pub struct Pod {
    /// Containers to run in the pod.
    pub containers: Vec<PodContainer>,

    /// Port mappings exposed by the pod.
    #[serde(default)]
    pub ports: Vec<PodPort>,
}

/// Definition of a pod's container
#[derive(Debug, Deserialize)]
pub struct PodContainer {
    /// Override container command.
    #[serde(default)]
    pub command: Option<Vec<String>>,

    /// Container image to run.
    pub image: String,

    /// Optional command run by `replidev deps initialise`.
    #[serde(default)]
    pub initialise: Option<Vec<String>>,

    /// Additional environment variables.
    #[serde(default)]
    pub env: BTreeMap<String, String>,

    /// Mount points to add to the container.
    #[serde(default)]
    pub mount: Vec<PodContainerMount>,

    /// Name of this container.
    pub name: String,
}

/// Definition of a pod's container
#[derive(Debug, Deserialize)]
pub struct PodContainerMount {
    #[serde(rename = "type")]
    pub mount_type: String,

    #[serde(default, flatten)]
    pub options: BTreeMap<String, String>,
}

/// Port mapping exposed by a pod.
#[derive(Debug, Deserialize)]
pub struct PodPort {
    /// Port to open on the host.
    pub host: usize,

    /// Port the pod will be listening on.
    #[serde(default)]
    pub pod: Option<usize>,
}
