use crate::conf::Conf;
use crate::Result;

use super::StopNodeOpt;

pub async fn run(args: &StopNodeOpt, conf: &Conf) -> Result<bool> {
    for node in &args.nodes {
        if crate::podman::pod_stop(conf, node).await.is_err() {
            println!(
                "--> Failed to stop {} pod, assuming it was not running",
                node
            );
        }
    }
    Ok(true)
}
