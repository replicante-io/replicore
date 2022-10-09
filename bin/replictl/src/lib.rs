use anyhow::Result;
use clap::Parser;
use slog::debug;
use slog::error;

mod apiclient;
mod commands;
mod context;
mod errors;
mod logging;
mod utils;

// Re-export errors so main can provide more accurate messages.
pub use apiclient::ApiNotFound;
pub use context::ScopeError;
pub use errors::ContextNotFound;
pub use errors::InvalidApply;

use commands::Command;
use context::ContextOpt;
use logging::LogOpt;

const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " [",
    env!("GIT_BUILD_HASH"),
    "; ",
    env!("GIT_BUILD_TAINT"),
    "]",
);

/// Replicante Core command line client.
#[derive(Debug, Parser)]
#[command(long_about = None)]
#[command(version = VERSION)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[command(flatten)]
    pub context: ContextOpt,

    #[command(flatten)]
    pub log: LogOpt,
}

pub fn run() -> Result<i32> {
    // Parse args and initialise logging.
    let cli = Cli::parse();
    let logger = logging::configure(&cli.log)?;
    debug!(
        logger,
        "replictl starting";
        "git-taint" => env!("GIT_BUILD_TAINT"),
        "git-commit" => env!("GIT_BUILD_HASH"),
        "version" => env!("CARGO_PKG_VERSION"),
    );

    // Set up a tokio runtime.
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("replictl-tokio-worker")
        .build()
        .expect("tokio runtime initialisation failed");
    let result = runtime.block_on(commands::execute(&logger, &cli));
    // Once done, ensure the runtime shuts down in a timely manner.
    // Note: this only effects blocking tasks and not futures.
    runtime.shutdown_timeout(std::time::Duration::from_millis(100));

    // Can finally exit.
    if result.is_err() {
        error!(logger, "replicli exiting with error");
    } else {
        debug!(logger, "replicli exiting successfully");
    }
    result
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    #[test]
    fn clap_integrity_check() {
        let command = crate::Cli::command();
        command.debug_assert();
    }
}
