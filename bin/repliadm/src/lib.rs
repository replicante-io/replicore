use clap::Arg;
use clap::ArgAction;
use clap::ArgMatches;
use clap::Command;
use slog::debug;
use slog::error;

mod commands;
mod error;
mod interfaces;
mod logging;
mod outcome;
mod utils;

pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;

use self::commands::coordinator;
use self::commands::validate;
use self::commands::versions;
use self::interfaces::Interfaces;
use self::logging::LogLevel;

const CLI_NAME: &str = "repliadm";
const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " [",
    env!("GIT_BUILD_HASH"),
    "; ",
    env!("GIT_BUILD_TAINT"),
    "]",
);

/// Process command line arguments and run the given command.
pub fn run() -> Result<()> {
    // Initialise clap CLI interface.
    let args = Command::new(CLI_NAME)
        .version(VERSION)
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .default_value("replicante.yaml")
                .num_args(1)
                .global(true)
                .help("Specifies the configuration file to use"),
        )
        .arg(
            Arg::new("log-level")
                .long("log-level")
                .value_name("LEVEL")
                .num_args(1)
                .value_parser(clap::value_parser!(LogLevel))
                .ignore_case(true)
                .global(true)
                .help("Specifies the logging verbosity"),
        )
        .arg(
            Arg::new("no-progress")
                .long("no-progress")
                .action(ArgAction::SetTrue)
                .global(true)
                .help("Do not show progress bars"),
        )
        .arg(
            Arg::new("progress-chunk")
                .long("progress-chunk")
                .value_name("CHUNK")
                .default_value("500")
                .num_args(1)
                .global(true)
                .help("Specifies how frequently to show progress messages"),
        )
        .subcommand(coordinator::command())
        .subcommand(validate::command())
        .subcommand(versions::command())
        .get_matches();

    // Initialise logging.
    let log_level = args.get_one::<LogLevel>("log-level")
        .cloned()
        .unwrap_or_default();
    let logger = logging::configure(log_level);
    debug!(logger, "repliadm starting"; "git-taint" => env!("GIT_BUILD_TAINT"));

    // Run the repliadm command.
    let interfaces = Interfaces::new(&args, logger.clone())?;
    let result = run_command(&args, &interfaces);
    if result.is_err() {
        error!(logger, "repliadm exiting with error"; "error" => true);
    } else {
        debug!(logger, "repliadm exiting with success"; "error" => false);
    }
    result
}

/// Switch the control flow to the requested command.
fn run_command(args: &ArgMatches, interfaces: &Interfaces) -> Result<()> {
    match args.subcommand_name() {
        Some(coordinator::COMMAND) => coordinator::run(args, interfaces),
        Some(validate::COMMAND) => validate::run(args, interfaces),
        Some(versions::COMMAND) => versions::run(args, interfaces),
        None => Err(ErrorKind::NoCommand(env!("CARGO_PKG_NAME").to_string()).into()),
        Some(name) => Err(ErrorKind::UnkownSubcommand(
            env!("CARGO_PKG_NAME").to_string(),
            name.to_string(),
        )
        .into()),
    }
}
