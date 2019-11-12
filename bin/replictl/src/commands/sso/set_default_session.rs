use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use slog::Logger;

use crate::sso::SessionStore;
use crate::ErrorKind;
use crate::Result;

pub const COMMAND: &str = "set-default-session";

pub fn command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(COMMAND)
        .about("Set the default SSO session used by future replictl invocations")
}

pub fn run<'a>(cli: &ArgMatches<'a>, _: &Logger) -> Result<()> {
    if cli.occurrences_of("session") == 0 {
        return Err(ErrorKind::CliOptMissing("session").into());
    }

    let mut sessions = SessionStore::load(cli)?;
    let name = cli
        .value_of("session")
        .expect("session name should be set by clap default")
        .to_string();
    println!(
        "Setting '{}' as the default SSO session for future commands",
        name
    );
    sessions.set_default_session(name);
    sessions.save(cli)
}
