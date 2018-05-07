extern crate clap;

#[macro_use]
extern crate error_chain;

use clap::App;
use clap::Arg;
use clap::ArgMatches;


mod commands;
mod errors;

pub use self::errors::Error;
pub use self::errors::ErrorKind;
pub use self::errors::ResultExt;
pub use self::errors::Result;

use self::commands::check;


/// Process command line arcuments and run the given command.
pub fn run() -> Result<()> {
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
             .help("Specifies the configuration file to use")
             .takes_value(true)
        )
        .subcommand(check::command())
        .get_matches();
    run_command(args)
}


/// Switch the control flow to the requested command.
fn run_command<'a>(args: ArgMatches<'a>) -> Result<()> {
    match args.subcommand_name() {
        Some(check::COMMAND) => check::run(args),
        None => Err("Need a command to run".into()),
        _ => Err("Received unrecognised command".into()),
    }
}
