#[macro_use]
extern crate clap;

#[macro_use]
extern crate error_chain;
extern crate indicatif;
extern crate serde_yaml;

#[macro_use]
extern crate slog;
extern crate slog_term;

extern crate replicante;
extern crate replicante_agent_discovery;

use clap::App;
use clap::Arg;
use clap::ArgMatches;


mod commands;
mod errors;
mod interfaces;
mod logging;
mod outcome;

pub use self::errors::Error;
pub use self::errors::ErrorKind;
pub use self::errors::ResultExt;
pub use self::errors::Result;

use self::commands::check;

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
             .help("Do not show progress bars")
        )
        .subcommand(check::command())
        .get_matches();

    // Initialise logging.
    let log_level = value_t!(args, "log-level", LogLevel).unwrap_or(LogLevel::default());
    let logger = logging::configure(log_level);
    debug!(logger, "replictl starting"; "git-taint" => env!("GIT_BUILD_TAINT"));

    // Run the replictl command.
    let interfaces = Interfaces::new(&args, logger.clone());
    let result = run_command(args, interfaces);
    match result.is_err() {
        false => debug!(logger, "replictl exiting with success"; "error" => false),
        true => error!(logger, "replictl exiting with error"; "error" => true),
    };
    result
}


/// Switch the control flow to the requested command.
fn run_command<'a>(args: ArgMatches<'a>, interfaces: Interfaces) -> Result<()> {
    match args.subcommand_name() {
        Some(check::COMMAND) => check::run(args, interfaces),
        None => Err("Need a command to run".into()),
        _ => Err("Received unrecognised command".into()),
    }
}
