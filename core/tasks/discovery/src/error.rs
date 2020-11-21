use std::fmt;

use failure::Backtrace;
use failure::Context;
use failure::Fail;

/// Error information returned by this crate.
#[derive(Debug)]
pub struct Error(Context<ErrorKind>);

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        self.0.get_context()
    }
}

impl Fail for Error {
    fn backtrace(&self) -> Option<&Backtrace> {
        self.0.backtrace()
    }

    fn cause(&self) -> Option<&dyn Fail> {
        self.0.cause()
    }

    fn name(&self) -> Option<&str> {
        self.kind().kind_name()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error(inner)
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error(Context::new(kind))
    }
}

/// Exhaustive list of possible errors emitted by this crate.
#[derive(Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "unable to deserialize task payload")]
    DeserializePayload,

    #[fail(display = "unable to emit {} event", _0)]
    EmitEvent(String),

    #[fail(
        display = "unable to fetch discovery record from backend {}.{}",
        _0, _1
    )]
    FetchCluster(String, String),

    #[fail(
        display = "unable to fetch {}.{} discovery record from primary store",
        _0, _1
    )]
    FetchDiscovery(String, String),

    #[fail(
        display = "unable to fetch {}.{} cluster settings record from primary store",
        _0, _1
    )]
    FetchSettings(String, String),

    #[fail(display = "unable to persist {}.{} discovery record", _0, _1)]
    PersistRecord(String, String),

    #[fail(display = "unable to persist {}.{} cluster settings", _0, _1)]
    PersistSettings(String, String),
}

impl ErrorKind {
    pub fn emit_event(code: &str) -> ErrorKind {
        ErrorKind::EmitEvent(code.to_string())
    }

    pub fn fetch_cluster(namespace: &str, name: &str) -> ErrorKind {
        ErrorKind::FetchCluster(namespace.to_string(), name.to_string())
    }

    pub fn fetch_discovery(namespace: &str, cluster_id: &str) -> ErrorKind {
        ErrorKind::FetchDiscovery(namespace.to_string(), cluster_id.to_string())
    }

    pub fn fetch_settings(namespace: &str, cluster_id: &str) -> ErrorKind {
        ErrorKind::FetchSettings(namespace.to_string(), cluster_id.to_string())
    }

    pub fn persist_record(namespace: &str, cluster_id: &str) -> ErrorKind {
        ErrorKind::PersistRecord(namespace.to_string(), cluster_id.to_string())
    }

    pub fn persist_settings(namespace: &str, cluster_id: &str) -> ErrorKind {
        ErrorKind::PersistSettings(namespace.to_string(), cluster_id.to_string())
    }

    fn kind_name(&self) -> Option<&str> {
        let name = match self {
            ErrorKind::EmitEvent(_) => "EmitEvent",
            ErrorKind::DeserializePayload => "DeserializePayload",
            ErrorKind::FetchCluster(_, _) => "FetchCluster",
            ErrorKind::FetchDiscovery(_, _) => "FetchDiscovery",
            ErrorKind::FetchSettings(_, _) => "FetchSettings",
            ErrorKind::PersistRecord(_, _) => "PersistRecord",
            ErrorKind::PersistSettings(_, _) => "PersistSettings",
        };
        Some(name)
    }
}

/// Short form alias for functions returning `Error`s.
pub type Result<T> = ::std::result::Result<T, Error>;
