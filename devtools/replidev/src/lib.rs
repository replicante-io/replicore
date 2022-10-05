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

pub fn run() -> anyhow::Result<i32> {
    // Parse CLI & conf.
    let args = Opt::from_args();
    let conf = conf::Conf::from_file()?;

    // Set up tokio runtime for all futures.
    // The AcitxWeb HttpServer can run in tokio native (even multi-threaded) unless it needs
    // actor-based features (such as web-sockets).
    // If the use of actix_rt::System becomes necessary a dedicated thread for actix-web becomes
    // the only option, with the questions about cross-runtime clients that may come with it.
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("replidev-tokio-worker")
        .build()
        .expect("tokio runtime init failed");

    // Run all commands inside the tokio runtime.
    let result = runtime.block_on(async {
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
