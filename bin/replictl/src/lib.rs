use clap::App;
use slog::debug;
use slog::error;

mod commands;
mod error;
mod logging;
mod sso;
mod utils;

pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;

pub const CLI_NAME: &str = "replictl";
const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " [",
    env!("GIT_BUILD_HASH"),
    "; ",
    env!("GIT_BUILD_TAINT"),
    "]",
);

/// Process command line arcuments and run the given command.
pub fn run() -> Result<()> {
    // Initialise clap CLI interface.
    let cli = App::new(CLI_NAME)
        .version(VERSION)
        .about(env!("CARGO_PKG_DESCRIPTION"));
    let cli = logging::configure_cli(cli);
    let cli = sso::configure_cli(cli);
    let cli = commands::configure_cli(cli);
    let cli = cli.get_matches();

    // Initialise global sub-systems.
    let logger = logging::configure(&cli)?;
    debug!(
        logger,
        "replictl starting";
        "git-taint" => env!("GIT_BUILD_TAINT"),
        "git-commit" => env!("GIT_BUILD_HASH"),
        "version" => env!("CARGO_PKG_VERSION"),
    );

    // Execute the command requested by the user.
    let result = commands::execute(&cli, &logger);
    if result.is_err() {
        error!(logger, "replicli exiting with error"; "error" => true);
    } else {
        debug!(logger, "replicli exiting with success"; "error" => false);
    }
    result
}
