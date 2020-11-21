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

    #[fail(display = "unable to fetch discovery record from backend")]
    FetchCluster,

    #[fail(display = "unable to fetch cluster settings record from primary store")]
    FetchSettings,

    #[fail(display = "unable to persist discovery record")]
    PersistRecord,

    #[fail(display = "unable to persist cluster settings")]
    PersistSettings,
}

impl ErrorKind {
    fn kind_name(&self) -> Option<&str> {
        let name = match self {
            ErrorKind::DeserializePayload => "DeserializePayload",
            ErrorKind::FetchCluster => "FetchCluster",
            ErrorKind::FetchSettings => "FetchSettings",
            ErrorKind::PersistRecord => "PersistRecord",
            ErrorKind::PersistSettings => "PersistSettings",
        };
        Some(name)
    }
}

/// Short form alias for functions returning `Error`s.
pub type Result<T> = ::std::result::Result<T, Error>;
