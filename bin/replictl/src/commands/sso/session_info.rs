use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use slog::Logger;

use crate::sso::SessionStore;
use crate::Error;
use crate::ErrorKind;
use crate::Result;

pub const COMMAND: &str = "session-info";

pub fn command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(COMMAND).about("Show information about an SSO session")
}

pub fn run<'a>(cli: &ArgMatches<'a>, _: &Logger) -> Result<()> {
    let sessions = SessionStore::load(cli)?;
    let session = sessions.active(cli).ok_or_else(|| {
        // No session found with the gven name.
        let name = sessions.active_name(cli);
        Error::from(ErrorKind::SessionNotFound(name))
    })?;

    // Show session information.
    println!("Replicante API address: {}", session.url);
    Ok(())
}
