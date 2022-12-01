use std::collections::BTreeMap;
use std::fs::File;

use failure::ResultExt;
use rand::Rng;

use crate::conf::Conf;
use crate::podman::Pod;
use crate::settings::Variables;
use crate::ErrorKind;
use crate::Result;

/// Arguments to a [`start_node`] call.
///
/// This allows the caller to define the node specification based on the context
/// (for example CLI vs Platform server) while reusing the actual node creation logic.
#[derive(Debug)]
pub struct StartNodeSpec<'a> {
    /// ID of the cluster to create the node in.
    pub cluster_id: &'a str,

    /// ID of the node being created.
    pub node_id: &'a str,

    /// Replidev project ID to attach to the node pod.
    pub project: String,

    /// The store software to provision.
    pub store: &'a str,

    /// The version of the store software to provision.
    ///
    /// Mainly for use by the Platform server, if set added to template variables.
    pub store_version: Option<&'a str>,

    /// Variables passed to the node pod template for customisation.
    pub variables: Variables,
}

/// Start a data store node from the given specification.
///
/// This function will only fail if pod creation fails,
/// if the pod is created but processes in it fail immediately this function does NOT fail.
pub async fn start_node(spec: StartNodeSpec<'_>, conf: &Conf) -> Result<()> {
    // Load node definition.
    let definition = format!("stores/{}/node.yaml", spec.store);
    let definition = File::open(definition).with_context(|_| ErrorKind::pod_not_found(spec.store))?;
    let pod: Pod = serde_yaml::from_reader(definition)
        .with_context(|_| ErrorKind::invalid_pod(spec.store))?;

    // Inject cluster & pod name annotations.
    let labels = {
        let mut labels = BTreeMap::new();
        labels.insert(
            "io.replicante.dev/play/cluster".to_string(),
            spec.cluster_id.to_string(),
        );
        labels.insert(
            "io.replicante.dev/project".to_string(),
            spec.project,
        );
        labels
    };

    // Extend template variables.
    let mut variables = spec.variables;
    variables.set("CLUSTER_ID", spec.cluster_id);
    if let Some(store_version) = spec.store_version {
        variables.set("STORE_VERSION", store_version);
    }

    // Start the node pod.
    crate::podman::pod_start(conf, pod, spec.node_id, labels, variables).await?;
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
