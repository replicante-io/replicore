use structopt::StructOpt;

use crate::conf::Conf;
use crate::Result;

/// Configuration related commands.
#[derive(Debug, StructOpt)]
pub enum CliOpt {
    /// Show the configured or detected local IP address.
    #[structopt(name = "ip")]
    ShowIp,
}

/// Configuration related commands.
pub async fn run(args: CliOpt, conf: Conf) -> Result<i32> {
    match args {
        CliOpt::ShowIp => show_ip(&conf),
    }
}

fn show_ip(conf: &Conf) -> Result<i32> {
    let ip = conf.podman_host_ip()?;
    println!("IP that will be used for podman-host alias: {}", ip);
    Ok(0)
}
