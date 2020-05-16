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
    // If an active session exists use it for defaults, otherwise use an "empty" session.
    let mut sessions = SessionStore::load(cli)?;
    let mut session = sessions.active(cli).unwrap_or_else(|| Session {
        ca_bundle: None,
        client_key: None,
        url: "".to_string(),
    });

    // Interact with the user to create/update the session.
    session.url = Input::new()
        .with_prompt("Replicante API address")
        .with_initial_text(&session.url)
        .interact()
        .with_context(|_| ErrorKind::UserInteraction)?;

    let ca_bundle = session.ca_bundle.clone().unwrap_or_else(|| "".to_string());
    let ca_bundle: String = Input::new()
        .with_prompt("(Optional) API CA certificate file")
        .with_initial_text(ca_bundle)
        .allow_empty(true)
        .interact()
        .with_context(|_| ErrorKind::UserInteraction)?;
    session.ca_bundle = match ca_bundle {
        path if path == "" => None,
        path if path.starts_with('/') => Some(path),
        path => {
            let current_dir = std::env::current_dir().expect("the current directory to be set");
            let current_dir = current_dir
                .as_path()
                .to_str()
                .expect("the current directory to be a valid path");
            let path = format!("{}/{}", current_dir, path);
            Some(path)
        }
    };

    let client_key = session.client_key.clone().unwrap_or_else(|| "".to_string());
    let client_key: String = Input::new()
        .with_prompt("(Optional) API client certificate bundle")
        .with_initial_text(client_key)
        .allow_empty(true)
        .interact()
        .with_context(|_| ErrorKind::UserInteraction)?;
    session.client_key = match client_key {
        path if path == "" => None,
        path if path.starts_with('/') => Some(path),
        path => {
            let current_dir = std::env::current_dir().expect("the current directory to be set");
            let current_dir = current_dir
                .as_path()
                .to_str()
                .expect("the current directory to be a valid path");
            let path = format!("{}/{}", current_dir, path);
            Some(path)
        }
    };

    // Update the sessions store with the new/updated login.
    let name = sessions.active_name(cli);
    sessions.upsert(name, session);
    sessions.save(cli)
}
