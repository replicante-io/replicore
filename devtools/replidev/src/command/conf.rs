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
pub fn run(args: CliOpt, conf: Conf) -> Result<bool> {
    match args {
        CliOpt::ShowIp => show_ip(&conf),
    }
}

fn show_ip(conf: &Conf) -> Result<bool> {
    let ip = conf.podman_host_ip()?;
    println!("IP that will be used for podman-host alias: {}", ip);
    Ok(true)
}
