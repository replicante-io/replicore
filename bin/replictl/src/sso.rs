use std::collections::BTreeMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::path::Path;

use clap::App;
use clap::Arg;
use clap::ArgMatches;
use failure::Fail;
use failure::ResultExt;
use serde::Deserialize;
use serde::Serialize;

use crate::utils::resolve_home;
use crate::ErrorKind;
use crate::Result;

const DEFAULT_SESSION_NAME: &str = "default";
const DEFAULT_SESSION_STORE: &str = "~/.replictl/credentials";

/// Information needed to access the Replicante API.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Session {
    /// Bundle of CA certificated to validate the API server with.
    #[serde(default)]
    pub ca_bundle: Option<String>,

    /// Client key and certificate PEM bundle for mutual TLS.
    #[serde(default)]
    pub client_key: Option<String>,

    /// URL to connect to the Replicante Core API servers.
    pub url: String,
}

/// Persistent collection of known sessions.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct SessionStore {
    /// Alias to the current active session.
    #[serde(rename = "<<active>>")]
    active: Option<String>,

    /// Collection of known sessions.
    #[serde(flatten)]
    sessions: BTreeMap<String, Session>,
}

impl SessionStore {
    /// Return the active session, if one is available.
    pub fn active<'a>(&self, cli: &ArgMatches<'a>) -> Option<Session> {
        let name = self.active_name(cli);
        self.sessions.get(&name).cloned()
    }

    /// Return the name of the active session.
    ///
    /// The name is looked up as follows:
    ///
    ///   * Use the session name from the CLI arguments, if `--session` was used.
    ///   * Use the `active` alias from the SessionStore, if it is set.
    ///   * As a fallback use `DEFAULT_SESSION_NAME`.
    pub fn active_name<'a>(&self, cli: &ArgMatches<'a>) -> String {
        // If --session is used force the given name.
        if cli.occurrences_of("session") != 0 {
            return cli
                .value_of("session")
                .expect("session name should be set by clap default")
                .to_string();
        }

        // Otherwise derive it from the `active` session.
        self.active
            .clone()
            .unwrap_or_else(|| DEFAULT_SESSION_NAME.to_string())
    }

    /// Load the session store from disk.
    pub fn load<'a>(cli: &ArgMatches<'a>) -> Result<SessionStore> {
        let file = cli
            .value_of("session-store")
            .expect("session store path should be set by clap default");
        let file = resolve_home(file)?;
        let reader = match File::open(&file) {
            Ok(file) => Some(file),
            Err(error) => {
                if error.kind() != std::io::ErrorKind::NotFound {
                    return Err(error.context(ErrorKind::FsOpen(file)).into());
                }
                None
            }
        };
        let reader = match reader {
            None => {
                return Ok(SessionStore {
                    active: None,
                    sessions: BTreeMap::new(),
                });
            }
            Some(reader) => reader,
        };
        let sessions =
            serde_yaml::from_reader(reader).with_context(|_| ErrorKind::SessionsDecode)?;
        Ok(sessions)
    }

    /// Iterate over stored session names.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.sessions.keys().map(|key| key.as_str())
    }

    /// Remove a session from the store.
    pub fn remove(&mut self, name: &str) {
        self.sessions.remove(name);
    }

    /// Write the session store to disk.
    ///
    /// If the path containing the credentials file does not exist it will be created.
    pub fn save<'a>(&self, cli: &ArgMatches<'a>) -> Result<()> {
        let file = cli
            .value_of("session-store")
            .expect("session store path should be set by clap default");
        let file = resolve_home(file)?;
        let path = Path::new(&file);

        // Create sessions store file parent directory if needed.
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir(parent).with_context(|_| {
                    let parent = parent
                        .to_str()
                        .expect("session store path to be UTF-8 encoded")
                        .to_string();
                    ErrorKind::FsMkDir(parent)
                })?;
            }
        }

        // Store the updated sessions store.
        let writer = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)
            .with_context(|_| ErrorKind::FsOpen(file.to_string()))?;
        serde_yaml::to_writer(writer, self).with_context(|_| ErrorKind::SessionsEncode)?;
        Ok(())
    }

    /// Set the default active session name for future calls.
    pub fn set_default_session(&mut self, active: String) {
        self.active = Some(active);
    }

    /// Insert of update a named `Session`.
    pub fn upsert(&mut self, name: String, session: Session) {
        self.sessions.insert(name, session);
    }
}

/// Configure the given `clap::App` with SSO related options.
pub fn configure_cli<'a, 'b>(cli: App<'a, 'b>) -> App<'a, 'b> {
    cli.arg(
        Arg::with_name("session")
            .long("session")
            .value_name("NAME")
            .takes_value(true)
            .default_value(DEFAULT_SESSION_NAME)
            .global(true)
            .help("Name of the SSO session to use"),
    )
    .arg(
        Arg::with_name("session-store")
            .long("session-store")
            .value_name("FILE")
            .takes_value(true)
            .default_value(DEFAULT_SESSION_STORE)
            .global(true)
            .help("Path to the sessions store "),
    )
}
