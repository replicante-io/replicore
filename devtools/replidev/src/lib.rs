use structopt::StructOpt;

mod command;
mod conf;
mod error;
mod podman;

use self::conf::Conf;

pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;

#[derive(Debug, StructOpt)]
#[structopt(name = "replidev", about = "Replicante Development Tool")]
struct CliOpt {
    /// The command to execute.
    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Manage Replicante Core dependencies.
    #[structopt(name = "deps")]
    Dependencies(self::command::deps::CliOpt),

    /// Manage Replicante Playgrounds nodes.
    #[structopt(name = "play")]
    Play,
}

pub fn run() -> Result<bool> {
    let args = CliOpt::from_args();
    let conf = self::conf::Conf::from_file()?;
    match args.command {
        Command::Dependencies(deps) => self::command::deps::run(deps, conf),
        Command::Play => panic!("TODO"),
    }
}
