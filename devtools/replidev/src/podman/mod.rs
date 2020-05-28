use std::collections::BTreeMap;

use serde::Deserialize;

mod build_cmd;
mod copy_cmd;
mod exec_cmd;
mod pod_inspect_cmd;
mod pod_ps_cmd;
mod pod_start_cmd;
mod pod_stop_cmd;
mod pull_cmd;
mod push_cmd;
mod run_cmd;
mod stop_cmd;
mod unshare_cmd;

pub use build_cmd::build;
pub use copy_cmd::copy;
pub use exec_cmd::exec;
pub use pod_inspect_cmd::pod_inspect;
pub use pod_ps_cmd::pod_ps;
pub use pod_start_cmd::pod_start;
pub use pod_stop_cmd::pod_stop;
pub use pull_cmd::pull;
pub use push_cmd::push;
pub use run_cmd::run;
pub use stop_cmd::stop;
pub use unshare_cmd::unshare;

/// Definition of a pod to start with podman.
#[derive(Clone, Debug, Deserialize)]
pub struct Pod {
    /// Containers to run in the pod.
    pub containers: Vec<PodContainer>,

    /// Port mappings exposed by the pod.
    #[serde(default)]
    pub ports: Vec<PodPort>,
}

/// Definition of a pod's container
#[derive(Clone, Debug, Deserialize)]
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

    /// Wait some seconds for the container to fully start.
    #[serde(default)]
    pub start_delay: Option<u64>,

    /// Container ulimits to set.
    #[serde(default)]
    pub ulimit: BTreeMap<String, String>,

    /// Optional user override.
    #[serde(default)]
    pub user: Option<String>,

    /// Optional working directory to set for the container.
    #[serde(default)]
    pub workdir: Option<String>,
}

/// Definition of a pod's container
#[derive(Clone, Debug, Deserialize)]
pub struct PodContainerMount {
    /// The type of mount operation to perform.
    #[serde(rename = "type")]
    pub mount_type: String,

    /// Addtional options passed to podman `--mount` command.
    #[serde(default, flatten)]
    pub options: BTreeMap<String, String>,

    /// Uiser ID to own the bind mounted sources.
    #[serde(default)]
    pub uid: Option<String>,
}

/// Port mapping exposed by a pod.
#[derive(Clone, Debug, Deserialize)]
pub struct PodPort {
    /// Port to open on the host.
    pub host: usize,

    /// Optional port name to inject the host port as a pod annotation.
    pub name: Option<String>,

    /// Port the pod will be listening on.
    #[serde(default)]
    pub pod: Option<usize>,

    /// List of protocols to bind the port to.
    #[serde(default)]
    pub protocols: Option<Vec<String>>,
}
