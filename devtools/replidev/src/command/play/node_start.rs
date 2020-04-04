use std::collections::BTreeMap;
use std::collections::HashSet;
use std::fs::File;
use std::net::TcpListener;

use failure::ResultExt;
use rand::Rng;

use crate::conf::Conf;
use crate::podman::Pod;
use crate::ErrorKind;
use crate::Result;

use super::NodeOpt;

pub fn run(args: &NodeOpt, conf: &Conf) -> Result<bool> {
    let name = random_name(8);
    let store = &args.store;
    let cluster_id = args
        .cluster_id
        .as_deref()
        .unwrap_or(store)
        .replace('/', "-");

    // Load node definition.
    let def = format!("stores/{}/node.yaml", store);
    let def = File::open(def).with_context(|_| ErrorKind::pod_not_found(store))?;
    let mut pod: Pod =
        serde_yaml::from_reader(def).with_context(|_| ErrorKind::invalid_pod(store))?;

    // Allocate random ports for any host == 0 definitions.
    let mut taken: HashSet<usize> = pod
        .ports
        .iter()
        .filter(|port| port.host != 0)
        .map(|port| port.host)
        .collect();
    for mut port in &mut pod.ports {
        if port.host != 0 {
            continue;
        }
        port.host = find_host_port(&taken);
        taken.insert(port.host);
    }

    // Inject cluster & pod name annotations.
    let labels = {
        let mut labels = BTreeMap::new();
        labels.insert(
            "io.replicante.dev/play/cluster".to_string(),
            cluster_id.clone(),
        );
        labels.insert(
            "io.replicante.dev/project".to_string(),
            conf.project.to_string(),
        );
        labels
    };

    // Prepare the node template environment.
    let paths = crate::settings::paths::PlayPod::new(store, &cluster_id, &name);
    let mut variables = crate::settings::Variables::new(conf, paths);
    variables.set("CLUSTER_ID", &cluster_id);

    // Start the node pod.
    println!(
        "--> Starting {} node {} for cluster {}",
        store, name, cluster_id
    );
    crate::podman::pod_start(conf, pod, name, labels, variables)?;
    Ok(true)
}

fn find_host_port(taken: &HashSet<usize>) -> usize {
    let port = (10000..60000).find(|port| {
        if taken.contains(port) {
            return false;
        }
        TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok()
    });
    port.expect("unable to find a usable host port")
}

fn random_name(len: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    let mut rng = rand::thread_rng();
    let name: String = (0..len)
        .map(|_| {
            let idx = rng.gen_range(0, CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    format!("play-node-{}", name)
}
