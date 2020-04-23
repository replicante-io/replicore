use std::io::BufRead;

use crate::conf::Conf;
use crate::Result;

use super::StopClusterOpt;
use super::StopNodeOpt;

pub fn run(args: &StopClusterOpt, conf: &Conf) -> Result<bool> {
    for cluster in &args.clusters {
        println!("--> Stopping cluster {}", cluster);
        let nodes = nodes_in_cluster(cluster, conf)?;
        if nodes.is_empty() {
            println!("--> Skipping cluster {} without running nodes", cluster);
            continue;
        }
        let node_stop_opts = StopNodeOpt { nodes };
        super::node_stop::run(&node_stop_opts, conf)?;
    }
    Ok(true)
}

/// Return a list of nodes in the given cluster.
fn nodes_in_cluster(cluster: &str, conf: &Conf) -> Result<Vec<String>> {
    let pod_names = crate::podman::pod_ps(
        conf,
        "{{ .Name }}",
        vec![
            &format!("label=io.replicante.dev/play/cluster={}", cluster),
            &format!("label=io.replicante.dev/project={}", conf.project),
        ],
    )?;
    let mut nodes = Vec::new();
    for name in pod_names.lines() {
        let name = name.expect("unable to read podman ps output");
        nodes.push(name);
    }
    Ok(nodes)
}
