use std::fmt;

use failure::Backtrace;
use failure::Context;
use failure::Fail;

use super::NodeId;

/// Error information returned by the `Coordinator` API in case of errors.
#[derive(Debug)]
pub struct Error(Context<ErrorKind>);

impl Error {
    pub fn kind(&self) -> ErrorKind {
        self.0.get_context().clone()
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error(Context::new(kind))
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error(inner)
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.0.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.0.backtrace()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// Exhaustive list of possible errors emitted by this crate.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "connection to coordinator failed")]
    BackendConnect,

    #[fail(display = "{} failed due to coordinator error", _0)]
    Backend(&'static str),

    #[fail(display = "failed to decode {}", _0)]
    Decode(&'static str),

    #[fail(display = "failed to encode {}", _0)]
    Encode(&'static str),

    #[fail(display = "election '{}' is no longer available", _0)]
    ElectionLost(String),

    #[fail(display = "election '{}' not found", _0)]
    ElectionNotFound(String),

    #[fail(display = "election '{}' already running", _0)]
    ElectionRunning(String),

    #[fail(display = "lock '{}' is already held by process '{}'", _0, _1)]
    LockHeld(String, NodeId),

    #[fail(display = "lock '{}' was lost", _0)]
    LockLost(String),

    #[fail(display = "lock '{}' not found", _0)]
    LockNotFound(String),

    #[fail(display = "lock '{}' is held by process '{}'", _0, _1)]
    LockNotHeld(String, NodeId),

    #[fail(display = "unable to spawn new thread for '{}'", _0)]
    SpawnThread(&'static str),
}

/// Short form alias for functions returning `Error`s.
pub type Result<T> = ::std::result::Result<T, Error>;
