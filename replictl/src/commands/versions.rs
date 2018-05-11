use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use error_chain::ChainedError;
use prometheus::Registry;

use replicante::Config;
use replicante::VERSION as REPLICANTE_VERSION;
use replicante_data_store::Validator;

use super::super::Interfaces;
use super::super::Result;
use super::super::ResultExt;


pub const COMMAND: &'static str = "versions";

lazy_static! {
    /// Version details for replictl.
    static ref VERSION: String = format!(
        "{} [{}; {}]",
        env!("CARGO_PKG_VERSION"), env!("GIT_BUILD_HASH"), env!("GIT_BUILD_TAINT")
    );
}


/// Collect information about various needed systems.
struct Versions {
    replicante: String,
    replictl: String,
    store: Result<String>,
}

impl Versions {
    /// Print all the collected information to stdout.
    pub fn show(&self) {
        println!("replictl: {}", self.replictl);
        println!("Replicante (statically determined): {}", self.replicante);
        let store = match self.store {
            Err(ref error) => error.display_chain().to_string(),
            Ok(ref store) => store.clone(),
        };
        println!("Store: {}", store);
    }
}


/// Configure the `replictl check` command parser.
pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("Reports version information for various systems")
}


/// Switch the control flow to the requested check command.
pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let logger = interfaces.logger();
    info!(logger, "Showing versions");

    // Find external systems version.
    let config = args.value_of("config").unwrap();
    let config = Config::from_file(config).chain_err(|| "Failed to load configuration")?;
    let store = store_version(&config, interfaces);

    // Display results.
    let versions = Versions {
        replicante: REPLICANTE_VERSION.clone(),
        replictl: VERSION.clone(),
        store,
    };
    versions.show();
    Ok(())
}

fn store_version(config: &Config, interfaces: &Interfaces) -> Result<String> {
    let logger = interfaces.logger();
    let registry = Registry::new();
    let store = Validator::new(config.storage.clone(), logger.clone(), &registry)?;
    let version = store.version()?;
    Ok(version)
}
