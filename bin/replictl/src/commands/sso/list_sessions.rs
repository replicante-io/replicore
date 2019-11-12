use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use slog::Logger;

use crate::sso::SessionStore;
use crate::Result;

pub const COMMAND: &str = "list-sessions";

pub fn command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(COMMAND).about("List all known SSO sessions")
}

pub fn run<'a>(cli: &ArgMatches<'a>, _: &Logger) -> Result<()> {
    let sessions = SessionStore::load(cli)?;
    let active = sessions.active_name(cli);

    println!("Available SSO sessions:");
    for name in sessions.names() {
        let mark = if name == active { " (active)" } else { "" };
        println!("  {}{}", name, mark);
    }

    Ok(())
}
