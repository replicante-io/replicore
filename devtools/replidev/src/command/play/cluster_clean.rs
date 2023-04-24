use anyhow::Result;

use crate::conf::Conf;

use super::CleanClusterOpt;

pub async fn run(args: &CleanClusterOpt, conf: &Conf) -> Result<i32> {
    for cluster in &args.clusters {
        let data = format!("./data/nodes/{}", cluster);
        println!("--> Clean data for {} (from {})", cluster, data);
        if args.common.confirm {
            crate::podman::unshare(conf, vec!["rm", "-r", &data]).await?;
        } else {
            println!("Skipping: you must --confirm deleting data");
        }
    }
    Ok(0)
}
