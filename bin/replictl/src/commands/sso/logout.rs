use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use slog::Logger;

use crate::sso::SessionStore;
use crate::Result;

pub const COMMAND: &str = "logout";

pub fn command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(COMMAND).about("Logout of an SSO session")
}

pub fn run<'a>(cli: &ArgMatches<'a>, _: &Logger) -> Result<()> {
    let mut sessions = SessionStore::load(cli)?;
    let name = sessions.active_name(cli);

    // If a session with this name is available perform logout.
    if sessions.active(cli).is_some() {
        // TODO: perform logout once SSO API actually exists.
    }

    // Clear the session for the session store.
    sessions.remove(&name);
    sessions.save(cli)?;
    println!("Logged out of SSO session '{}'", name);
    Ok(())
}
