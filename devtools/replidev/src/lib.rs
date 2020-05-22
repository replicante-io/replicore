use structopt::StructOpt;

mod command;
mod conf;
mod error;
mod podman;
mod settings;

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
    /// Configuration related commands.
    #[structopt(name = "conf")]
    Configuration(self::command::conf::CliOpt),

    /// Manage Replicante Core dependencies.
    #[structopt(name = "deps")]
    Dependencies(self::command::deps::CliOpt),

    /// Generate an HTTPS CA with client and server certificates.
    #[structopt(name = "gen-certs")]
    GenCerts(self::command::certs::CliOpt),

    /// Manage Replicante Playgrounds nodes.
    #[structopt(name = "play")]
    Play(self::command::play::CliOpt),
}

impl Command {
    fn need_actix_rt(&self) -> bool {
        match self {
            Self::Play(play) => play.need_actix_rt(),
            _ => false,
        }
    }
}

pub fn run() -> Result<i32> {
    // Parse CLI & conf.
    let args = CliOpt::from_args();
    let conf = self::conf::Conf::from_file()?;

    // Set up tokio runtime for all futures and a LocalSet for actix-web.
    let mut runtime = tokio::runtime::Builder::new()
        .enable_all()
        .thread_name("replidev-tokio-worker")
        .threaded_scheduler()
        .build()
        .expect("tokio runtime init failed");
    let local = tokio::task::LocalSet::new();

    // Create an actix runtime that uses the existing tokio runtime.
    // This is required by some commands to run the web server.
    if args.command.need_actix_rt() {
        local.spawn_local(actix_rt::System::run_in_tokio("replidev-actix", &local));
    }

    // Run all commands inside the tokio runtime.
    let result = local.block_on(&mut runtime, async {
        // TODO: convert everything into async commands.
        match args.command {
            Command::Configuration(cfg) => self::command::conf::run(cfg, conf).await,
            Command::Dependencies(deps) => self::command::deps::run(deps, conf).await,
            Command::GenCerts(certs) => self::command::certs::run(certs, conf).await,
            Command::Play(play) => self::command::play::run(play, conf).await,
        }
    });

    // Once done, ensure the runtime shuts down in a timely manner.
    // Note: this only effects blocking tasks and not futures.
    runtime.shutdown_timeout(std::time::Duration::from_millis(100));
    result
}
