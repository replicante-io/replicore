use crate::conf::Conf;
use crate::Result;

use super::StopNodeOpt;

pub fn run(args: &StopNodeOpt, conf: &Conf) -> Result<bool> {
    for node in &args.nodes {
        if crate::podman::pod_stop(conf, node).is_err() {
            println!(
                "--> Failed to stop {} pod, assuming it was not running",
                node
            );
        }
    }
    Ok(true)
}
