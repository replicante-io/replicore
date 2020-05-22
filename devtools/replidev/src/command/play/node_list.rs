use std::collections::BTreeMap;
use std::io::BufRead;

use failure::ResultExt;
use prettytable::cell;
use prettytable::row;
use serde::Deserialize;

use crate::Conf;
use crate::ErrorKind;
use crate::Result;

/// List running node pods.
///
/// Example output:
///   NODE                CLUSTER    STORE PORT  CLIENT PORT  AGENT PORT  STATUS   POD ID
///   play-node-rS3KQZOw  mongo-rs   10000       10000        10001       Running  206f19a3692f
///   play-node-Niu57N4O  zookeeper  10100       10101        10102       Stopped  817215a1fb8f
///   play-node-B6ZM7FWZ  postgres   10200       10200        -           Running  e72f080534c8
pub async fn run(conf: &Conf) -> Result<i32> {
    let nodes = list_nodes(conf).await?;
    let mut table = prettytable::Table::new();
    table.add_row(row![
        "NODE",
        "CLUSTER",
        "STORE PORT",
        "CLIENT PORT",
        "AGENT PORT",
        "STATUS",
        "POD ID",
    ]);
    for node in nodes {
        let pod_id = &node.id[0..12];
        let port_agent = node.port_agent.as_deref().unwrap_or("-");
        let port_client = node.port_client.as_ref().unwrap_or(&node.port_store);
        table.add_row(row![
            node.node,
            node.cluster,
            node.port_store,
            port_client,
            port_agent,
            node.status,
            pod_id,
        ]);
    }

    let format = prettytable::format::FormatBuilder::new()
        .column_separator(' ')
        .padding(0, 2)
        .build();
    table.set_format(format);
    table.printstd();
    Ok(0)
}

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
        let pod: PodRawInfo = serde_json::from_slice(&pod)
            .with_context(|_| ErrorKind::response_decode("podman inspect"))?;
        let cluster = pod
            .config
            .labels
            .get("io.replicante.dev/play/cluster")
            .expect("playground pod without cluster can't be returned here")
            .to_string();
        let port_agent = pod
            .config
            .labels
            .get("io.replicante.dev/port/agent")
            .cloned();
        let port_client = pod
            .config
            .labels
            .get("io.replicante.dev/port/client")
            .cloned();
        let port_store = pod
            .config
            .labels
            .get("io.replicante.dev/port/store")
            .expect("playground pod without store port can't be returned here")
            .to_string();
        let pod = PodInfo {
            cluster,
            id: pod.config.id,
            node: pod.config.name,
            port_agent,
            port_client,
            port_store,
            status: pod.state.status,
        };
        pods.push(pod);
    }

    Ok(pods)
}

/// Information about a node pod.
#[derive(Debug)]
pub struct PodInfo {
    pub cluster: String,
    pub id: String,
    pub node: String,
    pub port_agent: Option<String>,
    pub port_client: Option<String>,
    pub port_store: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
struct PodRawInfo {
    #[serde(rename = "Config")]
    config: PodRawInfoConfig,

    #[serde(rename = "State")]
    state: PodRawInfoState,
}

#[derive(Debug, Deserialize)]
struct PodRawInfoConfig {
    id: String,
    name: String,
    labels: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct PodRawInfoState {
    status: String,
}
