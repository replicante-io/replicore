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
    #[fail(display = "message with ID '{}' has not event code", _0)]
    EventHasNoCode(String),

    #[fail(display = "message with ID '{}' has an unexpected event code", _0)]
    EventUnkownCode(String),

    #[fail(display = "could not acknowledge message with ID '{}'", _0)]
    EventsStreamAck(String),

    #[fail(display = "could not follow the events stream to update the view DB")]
    EventsStreamFollow,

    #[fail(display = "failed to read {} from the view store", _0)]
    StoreRead(&'static str),

    #[fail(display = "failed to persist {} to the view store", _0)]
    StoreWrite(&'static str),

    #[fail(display = "failed to spawn view updater thread")]
    ThreadSpawn,
}

impl ErrorKind {
    fn kind_name(&self) -> Option<&str> {
        let name = match self {
            ErrorKind::EventHasNoCode(_) => "EventHasNoCode",
            ErrorKind::EventUnkownCode(_) => "EventUnkownCode",
            ErrorKind::EventsStreamAck(_) => "EventsStreamAck",
            ErrorKind::EventsStreamFollow => "EventsStreamFollow",
            ErrorKind::StoreRead(_) => "StoreRead",
            ErrorKind::StoreWrite(_) => "StoreWrite",
            ErrorKind::ThreadSpawn => "ThreadSpawn",
        };
        Some(name)
    }
}

/// Short form alias for functions returning `Error`s.
pub type Result<T> = ::std::result::Result<T, Error>;
