use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use failure::ResultExt;
use failure::err_msg;
use prometheus::Registry;

use replicante::Config;
use replicante::VERSION as REPLICANTE_VERSION;
use replicante_data_store::Validator;

use super::super::ErrorKind;
use super::super::Interfaces;
use super::super::Result;
use super::super::core::Client;


pub const COMMAND: &str = "versions";

lazy_static! {
    /// Version details for replictl.
    static ref VERSION: String = format!(
        "{} [{}; {}]",
        env!("CARGO_PKG_VERSION"), env!("GIT_BUILD_HASH"), env!("GIT_BUILD_TAINT")
    );
}


/// Collect information about various needed systems.
struct Versions {
    replicante_dynamic: Result<String>,
    replicante_static: String,
    replictl: String,
    store: Result<String>,
}

impl Versions {
    /// Print all the collected information to stdout.
    pub fn show(&self) {
        println!("replictl: {}", self.replictl);
        println!("Replicante (statically determined): {}", self.replicante_static);
        println!(
            "Replicante (dynamically determined): {}",
            value_or_error(&self.replicante_dynamic)
        );
        println!("Store: {}", value_or_error(&self.store));
    }
}


/// Configure the `replictl version` command parser.
pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("Reports version information for various systems")
}


/// Collect version information from the replicante API.
fn replicante_version<'a>(args: &ArgMatches<'a>) -> Result<String> {
    let client = Client::new(args)?;
    let version = client.version()?;
    Ok(format!("{} [{}; {}]", version.version, version.commit, version.taint))
}


/// Switch the control flow to the requested check command.
pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let logger = interfaces.logger();
    info!(logger, "Showing versions");

    // Find external systems version.
    let config = args.value_of("config").unwrap();
    let config = Config::from_file(config)
        .context(ErrorKind::Legacy(err_msg("failed to load configuration")))?;
    let replicante_dynamic = replicante_version(args);
    let store = store_version(&config, interfaces);

    // Display results.
    let versions = Versions {
        replicante_dynamic,
        replicante_static: REPLICANTE_VERSION.clone(),
        replictl: VERSION.clone(),
        store,
    };
    versions.show();
    Ok(())
}


/// Collect version information for the configured store.
fn store_version(config: &Config, interfaces: &Interfaces) -> Result<String> {
    let logger = interfaces.logger();
    let registry = Registry::new();
    let store = Validator::new(config.storage.clone(), logger.clone(), &registry)
        .with_context(|_| ErrorKind::AdminInit("store"))?;
    let version = store.version().with_context(|_| ErrorKind::FetchVersion("store"))?;
    Ok(version)
}


/// Returns the value of the result or a formatted error message.
fn value_or_error(result: &Result<String>) -> String {
    match *result {
        Err(ref error) => error.to_string().trim_right().to_string(),
        Ok(ref value) => value.clone(),
    }
}
