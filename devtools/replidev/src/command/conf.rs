use anyhow::Result;
use structopt::StructOpt;

use crate::conf::Conf;

/// Configuration related commands.
#[derive(Debug, StructOpt)]
pub enum Opt {
    /// Show the configured or detected local IP address.
    #[structopt(name = "ip")]
    ShowIp,
}

/// Configuration related commands.
pub async fn run(args: Opt, conf: Conf) -> Result<i32> {
    match args {
        Opt::ShowIp => show_ip(&conf),
    }
}

fn show_ip(conf: &Conf) -> Result<i32> {
    let ip = conf
        .podman_host_ip()
        .map_err(crate::error::wrap_for_anyhow)?;
    println!("IP that will be used for podman-host alias: {}", ip);
    Ok(0)
}
