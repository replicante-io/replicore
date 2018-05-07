#[macro_use]
extern crate clap;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate slog;
extern crate slog_term;

use clap::App;
use clap::Arg;
use clap::ArgMatches;

use slog::Logger;


mod commands;
mod errors;
mod logging;

pub use self::errors::Error;
pub use self::errors::ErrorKind;
pub use self::errors::ResultExt;
pub use self::errors::Result;

use self::commands::check;

use self::logging::LogLevel;


/// Process command line arcuments and run the given command.
pub fn run() -> Result<()> {
    // Initialise clap.
    let version = format!(
        "{} [{}; {}]",
        env!("CARGO_PKG_VERSION"), env!("GIT_BUILD_HASH"), env!("GIT_BUILD_TAINT")
    );
    let args = App::new("replictl")
        .version(version.as_ref())
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(Arg::with_name("config")
             .short("c")
             .long("config")
             .value_name("FILE")
             .default_value("replicante.yaml")
             .takes_value(true)
             .help("Specifies the configuration file to use")
        )
        .arg(Arg::with_name("log-level")
             .long("log-level")
             .value_name("LEVEL")
             .takes_value(true)
             .possible_values(&LogLevel::variants())
             .case_insensitive(true)
             .help("Specifies the logging verbosity")
        )
        .subcommand(check::command())
        .get_matches();

    // Initialise logging.
    let log_level = value_t!(args, "log-level", LogLevel).unwrap_or(LogLevel::default());
    let logger = logging::configure(log_level);
    debug!(logger, "replictl starting"; "git-taint" => env!("GIT_BUILD_TAINT"));

    // Run the replictl.
    let result = run_command(args, logger.clone());
    match result.is_err() {
        false => info!(logger, "Shutdown: replictl exiting with success"; "error" => false),
        true => error!(logger, "Shutdown: replictl exiting with error"; "error" => true),
    };
    result
}


/// Switch the control flow to the requested command.
fn run_command<'a>(args: ArgMatches<'a>, logger: Logger) -> Result<()> {
    match args.subcommand_name() {
        Some(check::COMMAND) => check::run(args, logger),
        None => Err("Need a command to run".into()),
        _ => Err("Received unrecognised command".into()),
    }
}
