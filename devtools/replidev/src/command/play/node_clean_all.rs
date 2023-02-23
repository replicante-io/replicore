use anyhow::Result;

use crate::Conf;

use super::CleanCommonOpt;

pub async fn run(args: &CleanCommonOpt, conf: &Conf) -> Result<i32> {
    let data = "./data/nodes/";
    println!("--> Clean data for all nodes (from {})", data);
    if args.confirm {
        crate::podman::unshare(conf, vec!["rm", "-r", data]).await?;
    } else {
        println!("Skipping: you must --confirm deleting data");
    }
    Ok(0)
}
