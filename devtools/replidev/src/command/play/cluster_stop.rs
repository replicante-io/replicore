use std::io::BufRead;
//use tokio::io::BufRead;

use crate::conf::Conf;
use crate::Result;

use super::StopClusterOpt;
use super::StopNodeOpt;

pub async fn run(args: &StopClusterOpt, conf: &Conf) -> Result<i32> {
    for cluster in &args.clusters {
        println!("--> Stopping cluster {}", cluster);
        let nodes = nodes_in_cluster(cluster, conf).await?;
        if nodes.is_empty() {
            println!("--> Skipping cluster {} without running nodes", cluster);
            continue;
        }
        let node_stop_opts = StopNodeOpt { nodes };
        super::node_stop::run(&node_stop_opts, conf).await?;
    }
    Ok(0)
}

/// Return a list of nodes in the given cluster.
async fn nodes_in_cluster(cluster: &str, conf: &Conf) -> Result<Vec<String>> {
    let pod_names = crate::podman::pod_ps(
        conf,
        "{{ .Name }}",
        vec![
            &format!("label=io.replicante.dev/play/cluster={}", cluster),
            &format!("label=io.replicante.dev/project={}", conf.project),
        ],
    )
    .await?;
    let mut nodes = Vec::new();
    for name in pod_names.lines() {
        let name = name.expect("unable to read podman ps output");
        nodes.push(name);
    }
    Ok(nodes)
}
