use clap::Arg;
use clap::ArgAction;
use clap::ArgMatches;
use failure::ResultExt;
use slog::Logger;

use replicante::Config;
use replicante_service_coordinator::Admin as CoordinatorAdmin;
use replicante_store_primary::admin::Admin as PrimaryStoreAdmin;
use replicante_store_view::admin::Admin as ViewStoreAdmin;

use crate::ErrorKind;
use crate::Result;

/// Initialise the coordinator admin interface.
pub fn coordinator_admin(args: &ArgMatches, logger: Logger) -> Result<CoordinatorAdmin> {
    let config = load_config(args)?;
    let admin = CoordinatorAdmin::new(config.coordinator, logger)
        .with_context(|_| ErrorKind::AdminInit("coordinator"))?;
    Ok(admin)
}

/// Load Replicante Core configuration.
pub fn load_config(args: &ArgMatches) -> Result<Config> {
    let file = args
        .get_one::<String>("config")
        .expect("CLI argument --config is required");
    let config = Config::from_file(file).with_context(|_| ErrorKind::ConfigLoad)?;
    Ok(config)
}

/// Initialise the primary store admin interface.
pub fn primary_store_admin(args: &ArgMatches, logger: Logger) -> Result<PrimaryStoreAdmin> {
    let config = load_config(args)?;
    let admin = PrimaryStoreAdmin::make(config.storage.primary, logger)
        .with_context(|_| ErrorKind::AdminInit("primary store"))?;
    Ok(admin)
}

/// Return an "I take responsibility" CLI argument flag.
pub fn take_responsibility_arg() -> Arg {
    Arg::new("take-responsibility")
        .long("I-take-responsibility-for-this-action")
        .help("Acknowledges the desire to perform the operation")
        .action(ArgAction::SetTrue)
}

/// Initialise the view store admin interface.
pub fn view_store_admin(args: &ArgMatches, logger: Logger) -> Result<ViewStoreAdmin> {
    let config = load_config(args)?;
    let admin = ViewStoreAdmin::make(config.storage.view, logger)
        .with_context(|_| ErrorKind::AdminInit("view store"))?;
    Ok(admin)
}
