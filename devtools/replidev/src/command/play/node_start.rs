use crate::conf::Conf;
use crate::platform::node_start;
use crate::Result;

use super::StartNodeOpt;

pub async fn run(args: &StartNodeOpt, conf: &Conf) -> Result<i32> {
    let node_id: String = args
        .node_name
        .clone()
        .unwrap_or_else(|| node_start::random_node_id(8));
    let cluster_id = args
        .cluster_id
        .as_deref()
        .unwrap_or(&args.store)
        .replace('/', "-");

    // Prepare the node template environment.
    let paths = crate::settings::paths::PlayPod::new(&args.store, &cluster_id, &node_id);
    let mut variables = crate::settings::Variables::new(conf, paths)?;
    variables
        .set_cli_vars(&args.vars)?
        .set_cli_var_files(&args.var_files)?;

    // Start a new node pod.
    let start_node_spec = node_start::StartNodeSpec {
        cluster_id: &cluster_id,
        node_id: &node_id,
        project: conf.project.to_string(),
        store: &args.store,
        store_version: None,
        variables,
    };
    node_start::start_node(start_node_spec, conf).await?;

    println!(
        "--> Started {} node {} for cluster {}",
        &args.store, node_id, cluster_id
    );
    Ok(0)
}
