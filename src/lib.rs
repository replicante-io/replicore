extern crate clap;

#[macro_use]
extern crate error_chain;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

use clap::App;
use clap::Arg;


mod config;
mod errors;

use self::config::Config;

pub use self::errors::Error;
pub use self::errors::ErrorKind;
pub use self::errors::ResultExt;
pub use self::errors::Result;


#[doc(hidden)]
pub fn run() -> Result<()> {
    // Initialise and parse command line arguments.
    let version = format!(
        "{} [{}; {}]",
        env!("CARGO_PKG_VERSION"), env!("GIT_BUILD_HASH"), env!("GIT_BUILD_TAINT")
    );
    let cli_args = App::new("Replicante Core")
        .version(&version[..])
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(Arg::with_name("config")
             .short("c")
             .long("config")
             .value_name("FILE")
             .default_value("replicante.yaml")
             .help("Specifies the configuration file to use")
             .takes_value(true)
        )
        .get_matches();

    // TODO: Log initialisation start.

    // Load configuration.
    let config_location = cli_args.value_of("config").unwrap();
    let config = Config::from_file(config_location.clone())
        .chain_err(|| format!("Failed to load configuration: {}", config_location))?;

    // TODO: Wait for all threads to exit.
    println!("Main crate entrypoint");
    Ok(())
}
