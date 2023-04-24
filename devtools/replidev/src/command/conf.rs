use anyhow::Result;
use clap::Subcommand;

use crate::conf::Conf;

/// Configuration related commands.
#[derive(Debug, Subcommand)]
pub enum Opt {
    /// Show the configured or detected local IP address.
    #[command(name = "ip")]
    ShowIp,
}

/// Configuration related commands.
pub async fn run(args: Opt, conf: Conf) -> Result<i32> {
    match args {
        Opt::ShowIp => show_ip(&conf),
    }
}

fn show_ip(conf: &Conf) -> Result<i32> {
    let ip = conf.podman_host_ip()?;
    println!("IP detected for the podman host: {}", ip);
    Ok(0)
}
