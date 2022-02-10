use crate::conf::Conf;
use crate::settings::paths::Paths;
use crate::Result;

use super::CleanNodeOpt;

pub async fn run(args: &CleanNodeOpt, conf: &Conf) -> Result<i32> {
    for node in &args.nodes {
        let paths = crate::settings::paths::PlayPod::new("<unkown>", &args.cluster, node);
        let data = paths.data();
        println!(
            "--> Clean data for {}'s {} (from {})",
            args.cluster, node, data
        );
        if args.common.confirm {
            crate::podman::unshare(conf, vec!["rm", "-r", data]).await?;
        } else {
            println!("Skipping: you must --confirm deleting data");
        }
    }
    Ok(0)
}
