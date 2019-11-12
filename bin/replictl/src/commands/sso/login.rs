use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use dialoguer::Input;
use failure::ResultExt;
use slog::Logger;

use crate::sso::Session;
use crate::sso::SessionStore;
use crate::ErrorKind;
use crate::Result;

pub const COMMAND: &str = "login";

pub fn command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(COMMAND).about("Login to a Replicante instance")
}

pub fn run<'a>(cli: &ArgMatches<'a>, _: &Logger) -> Result<()> {
    let mut sessions = SessionStore::load(cli)?;
    // If an active session exists print a message suggesting logout.
    // TODO: once sessions are more then just a URL replace println! with URL reuse.
    if sessions.active(cli).is_some() {
        let name = sessions.active_name(cli);
        println!("An SSO session name '{}' already exists.", name);
        println!("To update it logout first with `replictl sso logout`");
        return Ok(());
    }

    let url: String = Input::new()
        .with_prompt("Replicante API address")
        .interact()
        .with_context(|_| ErrorKind::UserInteraction)?;
    let session = Session { url };
    let name = sessions.active_name(cli);
    sessions.upsert(name, session);
    sessions.save(cli)
}
