use crate::conf::Conf;
use crate::Result;

use super::CleanClusterOpt;

pub fn run(args: &CleanClusterOpt, conf: &Conf) -> Result<bool> {
    for cluster in &args.clusters {
        let data = format!("./data/nodes/{}", cluster);
        println!("--> Clean data for {} (from {})", cluster, data);
        if args.common.confirm {
            crate::podman::unshare(conf, vec!["rm", "-r", &data])?;
        } else {
            println!("Skipping: you must --confirm deleting data");
        }
    }
    Ok(true)
}
