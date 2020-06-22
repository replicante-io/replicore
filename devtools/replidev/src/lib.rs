use structopt::StructOpt;

pub mod error;

mod command;
mod conf;
mod podman;
mod settings;

use conf::Conf;

pub use error::Error;
pub use error::ErrorKind;
pub use error::Result;

#[derive(Debug, StructOpt)]
#[structopt(name = "replidev", about = "Replicante Development Tool")]
enum Opt {
    /// Run the given cargo command in all workspaces.
    #[structopt(name = "cargo")]
    Cargo(command::cargo::Opt),

    /// Configuration related commands.
    #[structopt(name = "conf")]
    Configuration(command::conf::Opt),

    /// Run curl, adding client certificates when projects have them.
    #[structopt(name = "curl")]
    Curl(command::curl::Opt),

    /// Manage Replicante Core dependencies.
    #[structopt(name = "deps")]
    Dependencies(command::deps::Opt),

    /// Generate an HTTPS CA with client and server certificates.
    #[structopt(name = "gen-certs")]
    GenCerts(command::certs::Opt),

    /// Manage Replicante Playgrounds nodes.
    #[structopt(name = "play")]
    Play(command::play::Opt),

    /// Mange Replicante projects release tasks.
    #[structopt(name = "release")]
    Release(command::release::Opt),
}

impl Opt {
    fn need_actix_rt(&self) -> bool {
        match self {
            Self::Play(play) => play.need_actix_rt(),
            _ => false,
        }
    }
}

pub fn run() -> anyhow::Result<i32> {
    // Parse CLI & conf.
    let args = Opt::from_args();
    let conf = conf::Conf::from_file()?;

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
    if args.need_actix_rt() {
        local.spawn_local(actix_rt::System::run_in_tokio("replidev-actix", &local));
    }

    // Run all commands inside the tokio runtime.
    let result = local.block_on(&mut runtime, async {
        match args {
            Opt::Cargo(cargo) => command::cargo::run(cargo, &conf).await,
            Opt::Configuration(cfg) => command::conf::run(cfg, conf).await,
            Opt::Curl(cfg) => command::curl::run(cfg, conf).await,
            Opt::Dependencies(deps) => command::deps::run(deps, conf).await,
            Opt::GenCerts(certs) => command::certs::run(certs, conf).await,
            Opt::Play(play) => command::play::run(play, conf).await,
            Opt::Release(release) => command::release::run(release, conf).await,
        }
    });

    // Once done, ensure the runtime shuts down in a timely manner.
    // Note: this only effects blocking tasks and not futures.
    runtime.shutdown_timeout(std::time::Duration::from_millis(100));
    result
}
