use std::collections::BTreeMap;

use anyhow::Result;
use rand::Rng;

use crate::conf::Conf;

/// Arguments to a [`start_node`] call.
///
/// This allows the caller to define the node specification based on the context
/// (for example CLI vs Platform server) while reusing the actual node creation logic.
#[derive(Debug)]
pub struct StartNodeSpec<'a> {
    /// ID of the cluster to create the node in.
    pub cluster_id: &'a str,

    /// Node class, if set by node provisioning.
    pub node_class: Option<&'a str>,

    /// ID of the node being created.
    pub node_id: &'a str,

    /// Node group, for nodes provisioned using the Platform API.
    pub node_group: Option<&'a str>,

    /// Pod definition to start the node with.
    pub pod: crate::podman::Pod,

    /// Replidev project ID to attach to the node pod.
    pub project: String,

    /// The store software to provision.
    pub store: &'a str,
}

/// Start a data store node from the given specification.
///
/// This function will only fail if pod creation fails,
/// if the pod is created but processes in it fail immediately this function does NOT fail.
pub async fn start_node(spec: StartNodeSpec<'_>, conf: &Conf) -> Result<()> {
    // Inject cluster & pod name annotations.
    let labels = {
        let mut labels = BTreeMap::new();
        labels.insert(
            "io.replicante.dev/play/cluster".to_string(),
            spec.cluster_id.into(),
        );
        labels.insert("io.replicante.dev/project".to_string(), spec.project);
        if let Some(node_class) = spec.node_class {
            labels.insert(
                "io.replicante.dev/play/class".to_string(),
                node_class.to_string(),
            );
        }
        if let Some(node_group) = spec.node_group {
            labels.insert(
                "io.replicante.dev/play/group".to_string(),
                node_group.to_string(),
            );
        }
        labels
    };

    // Define variables needed for pod_start to work.
    // These are different to provisioning attributes which are Platform specific.
    let paths = crate::settings::paths::PlayPod::new(spec.store, spec.cluster_id, spec.node_id);
    let variables = crate::settings::Variables::new(conf, paths);

    // Start the pod.
    crate::podman::pod_start(conf, spec.pod, spec.node_id, labels, variables).await?;
    Ok(())
}

/// Generate a random node ID.
pub fn random_node_id(len: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    let mut rng = rand::thread_rng();
    let name: String = (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    format!("play-node-{}", name)
}
