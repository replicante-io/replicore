use crate::Conf;
use crate::Result;

use super::CleanCommonOpt;

pub fn run(args: &CleanCommonOpt, conf: &Conf) -> Result<bool> {
    let data = "./data/nodes/";
    println!("--> Clean data for all nodes (from {})", data);
    if args.confirm {
        crate::podman::unshare(conf, vec!["rm", "-r", &data])?;
    } else {
        println!("Skipping: you must --confirm deleting data");
    }
    Ok(true)
}
