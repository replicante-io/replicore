use std::collections::BTreeMap;
use std::fs::File;

use failure::ResultExt;
use rand::Rng;

use crate::conf::Conf;
use crate::podman::Pod;
use crate::ErrorKind;
use crate::Result;

use super::StartNodeOpt;

pub async fn run(args: &StartNodeOpt, conf: &Conf) -> Result<i32> {
    let name: String = args.node_name.clone().unwrap_or_else(|| random_name(8));
    let store = &args.store;
    let cluster_id = args
        .cluster_id
        .as_deref()
        .unwrap_or(store)
        .replace('/', "-");

    // Load node definition.
    let def = format!("stores/{}/node.yaml", store);
    let def = File::open(def).with_context(|_| ErrorKind::pod_not_found(store))?;
    let pod: Pod = serde_yaml::from_reader(def).with_context(|_| ErrorKind::invalid_pod(store))?;

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
    let mut variables = crate::settings::Variables::new(conf, paths)?;
    variables
        .set("CLUSTER_ID", cluster_id.as_str())
        .set_cli_vars(&args.vars)?
        .set_cli_var_files(&args.var_files)?;

    // Start the node pod.
    println!(
        "--> Starting {} node {} for cluster {}",
        store, name, cluster_id
    );
    crate::podman::pod_start(conf, pod, name, labels, variables).await?;
    Ok(0)
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
