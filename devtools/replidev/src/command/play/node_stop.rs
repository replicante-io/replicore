use anyhow::Result;

use crate::conf::Conf;

use super::StopNodeOpt;

pub async fn run(args: &StopNodeOpt, conf: &Conf) -> Result<i32> {
    for node in &args.nodes {
        if crate::podman::pod_stop(conf, node).await.is_err() {
            println!(
                "--> Failed to stop {} pod, assuming it was not running",
                node
            );
        }
    }
    Ok(0)
}
