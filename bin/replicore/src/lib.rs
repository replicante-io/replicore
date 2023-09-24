//! Combine individual logical units to initialise and run a Replicante Core process.
use anyhow::Result;
use clap::Parser;

use replicore_conf::Conf;

mod cmd;
mod init;

pub use self::cmd::Cli;

/// Initialise the replicore process and invoke a command implementation.
pub async fn execute(cli: Cli, conf: Conf) -> Result<()> {
    match cli.command {
        cmd::Command::Server => cmd::server::run(cli, conf).await,
        cmd::Command::Sync => cmd::sync::run(cli, conf).await,
    }
}

/// Initialise the async runtime for the process and invoke [`execute`].
pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let conf = replicore_conf::load(&cli.config)?;
    conf.runtime
        .tokio
        .clone()
        .into_runtime()
        .expect("failed tokio runtime initialisation")
        .block_on(execute(cli, conf))
}
