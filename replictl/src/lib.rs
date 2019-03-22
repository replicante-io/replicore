#[macro_use]
extern crate clap;
#[macro_use]
extern crate failure;
extern crate failure_derive;

#[macro_use]
extern crate lazy_static;
extern crate prometheus;
extern crate reqwest;

extern crate serde_yaml;
#[macro_use]
extern crate slog;
extern crate slog_term;

extern crate replicante;
extern crate replicante_agent_discovery;
extern crate replicante_coordinator;
extern crate replicante_data_models;
extern crate replicante_data_store;
extern crate replicante_streams_events;
extern crate replicante_tasks;
extern crate replicante_util_failure;


use clap::App;
use clap::Arg;
use clap::ArgMatches;


mod commands;
mod core;
mod error;
mod interfaces;
mod logging;
mod outcome;

pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;

use self::commands::check;
use self::commands::coordinator;
use self::commands::versions;

use self::interfaces::Interfaces;
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
             .global(true)
             .help("Specifies the configuration file to use")
        )
        .arg(Arg::with_name("log-level")
             .long("log-level")
             .value_name("LEVEL")
             .takes_value(true)
             .possible_values(&LogLevel::variants())
             .case_insensitive(true)
             .global(true)
             .help("Specifies the logging verbosity")
        )
        .arg(Arg::with_name("no-progress")
             .long("no-progress")
             .global(true)
             .help("Do not show progress bars")
        )
        .arg(Arg::with_name("progress-chunk")
             .long("progress-chunk")
             .value_name("CHUNK")
             .default_value("500")
             .takes_value(true)
             .global(true)
             .help("Specifies how frequently to show progress messages")
        )
        .arg(Arg::with_name("url")
             .long("url")
             .value_name("URL")
             .default_value("http://localhost:16016/")
             .takes_value(true)
             .global(true)
             .help("Specifies the URL of the Replicante API to use")
        )
        .subcommand(check::command())
        .subcommand(coordinator::command())
        .subcommand(versions::command())
        .get_matches();

    // Initialise logging.
    let log_level = value_t!(args, "log-level", LogLevel).unwrap_or_default();
    let logger = logging::configure(log_level);
    debug!(logger, "replictl starting"; "git-taint" => env!("GIT_BUILD_TAINT"));

    // Run the replictl command.
    let interfaces = Interfaces::new(&args, logger.clone())?;
    let result = run_command(&args, &interfaces);
    if result.is_err() {
        error!(logger, "replictl exiting with error"; "error" => true);
    } else {
        debug!(logger, "replictl exiting with success"; "error" => false);
    }
    result
}


/// Switch the control flow to the requested command.
fn run_command(args: &ArgMatches, interfaces: &Interfaces) -> Result<()> {
    match args.subcommand_name() {
        Some(check::COMMAND) => check::run(args, interfaces),
        Some(coordinator::COMMAND) => coordinator::run(args, interfaces),
        Some(versions::COMMAND) => versions::run(args, interfaces),
        None => Err(ErrorKind::NoCommand("replictl").into()),
        Some(name) => Err(ErrorKind::UnkownSubcommand("replictl", name.to_string()).into()),
    }
}
