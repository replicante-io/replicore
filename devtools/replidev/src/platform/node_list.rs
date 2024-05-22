use std::collections::BTreeMap;
use std::io::BufRead;

use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;

use crate::podman::Error;
use crate::Conf;

/// Fetch all node pods and their information.
pub async fn list_nodes(conf: &Conf) -> Result<Vec<PodInfo>> {
    // Find running node pod IDs.
    let pod_ids = crate::podman::pod_ps(
        conf,
        "{{ .ID }}",
        vec![
            "label=io.replicante.dev/play/cluster",
            "label=io.replicante.dev/port/store",
            &format!("label=io.replicante.dev/project={}", conf.project),
        ],
    )
    .await?;

    // Inspect pods to get all needed attributes.
    let mut pods: Vec<PodInfo> = Vec::new();
    for pod_id in pod_ids.lines() {
        let pod_id = pod_id.expect("unable to read podman ps output");
        let pod = crate::podman::pod_inspect(conf, &pod_id).await?;
        let mut pod: Vec<PodRawInfo> =
            serde_json::from_slice(&pod).with_context(|| Error::pod_not_valid(pod_id))?;
        let pod = pod.pop().expect("inspected pod by ID not found");
        let cluster = pod
            .labels
            .get("io.replicante.dev/play/cluster")
            .expect("playground pod without cluster can't be returned here")
            .to_string();
        let node_class = pod.labels.get("io.replicante.dev/play/class").cloned();
        let node_group = pod.labels.get("io.replicante.dev/play/group").cloned();
        let port_agent = pod.labels.get("io.replicante.dev/port/agent").cloned();
        let port_client = pod.labels.get("io.replicante.dev/port/client").cloned();
        let port_store = pod
            .labels
            .get("io.replicante.dev/port/store")
            .expect("playground pod without store port can't be returned here")
            .to_string();
        let pod = PodInfo {
            cluster,
            id: pod.id,
            node: pod.name,
            node_class,
            node_group,
            port_agent,
            port_client,
            port_store,
            status: pod.state,
        };
        pods.push(pod);
    }

    Ok(pods)
}

/// Information about a node pod.
#[derive(Debug)]
pub struct PodInfo {
    /// Cluster ID the pod belongs to.
    pub cluster: String,

    /// ID of the pod.
    pub id: String,

    /// Node ID.
    pub node: String,

    /// Node class, if set by node provisioning.
    pub node_class: Option<String>,

    /// Node group, for nodes provisioned using the Platform API.
    pub node_group: Option<String>,

    /// Port the agent process is listening on.
    pub port_agent: Option<String>,

    /// Port the datastore is listening for clients on.
    pub port_client: Option<String>,

    /// Port the datastore is listening for other nodes on.
    pub port_store: String,

    /// Status of the pod.
    pub status: String,
}

#[derive(Debug, Deserialize)]
struct PodRawInfo {
    /// ID of the pod.
    #[serde(rename = "Id")]
    id: String,

    /// Name of the pod.
    #[serde(rename = "Name")]
    name: String,

    /// Labels attached to the pod.
    #[serde(rename = "Labels")]
    labels: BTreeMap<String, String>,

    /// State of the pod.
    #[serde(rename = "State")]
    state: String,
}
