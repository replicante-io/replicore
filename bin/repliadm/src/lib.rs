use clap::value_t;
use clap::App;
use clap::Arg;
use clap::ArgMatches;
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

/// Process command line arcuments and run the given command.
pub fn run() -> Result<()> {
    // Initialise clap.
    let version = format!(
        "{} [{}; {}]",
        env!("CARGO_PKG_VERSION"),
        env!("GIT_BUILD_HASH"),
        env!("GIT_BUILD_TAINT"),
    );
    let args = App::new(CLI_NAME)
        .version(version.as_ref())
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .default_value("replicante.yaml")
                .takes_value(true)
                .global(true)
                .help("Specifies the configuration file to use"),
        )
        .arg(
            Arg::with_name("log-level")
                .long("log-level")
                .value_name("LEVEL")
                .takes_value(true)
                .possible_values(&LogLevel::variants())
                .case_insensitive(true)
                .global(true)
                .help("Specifies the logging verbosity"),
        )
        .arg(
            Arg::with_name("no-progress")
                .long("no-progress")
                .global(true)
                .help("Do not show progress bars"),
        )
        .arg(
            Arg::with_name("progress-chunk")
                .long("progress-chunk")
                .value_name("CHUNK")
                .default_value("500")
                .takes_value(true)
                .global(true)
                .help("Specifies how frequently to show progress messages"),
        )
        .subcommand(coordinator::command())
        .subcommand(validate::command())
        .subcommand(versions::command())
        .get_matches();

    // Initialise logging.
    let log_level = value_t!(args, "log-level", LogLevel).unwrap_or_default();
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
        Some(name) => Err(ErrorKind::UnkownSubcommand(env!("CARGO_PKG_NAME").to_string(), name.to_string()).into()),
    }
}
