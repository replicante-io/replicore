use std::fmt;

use failure::Backtrace;
use failure::Context;
use failure::Fail;

/// Error information returned by functions in case of errors.
#[derive(Debug)]
pub struct Error(Context<ErrorKind>);

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        self.0.get_context()
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

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.0.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.0.backtrace()
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

/// Exhaustive list of possible errors emitted by this crate.
#[derive(Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "error connecting to agent {}", _0)]
    AgentConnect(String),

    #[fail(display = "error fetching {} from agent {}", _0, _1)]
    AgentRead(&'static str, String),

    #[fail(
        display = "expected cluster display name '{}' but found '{}' for node with ID '{}'",
        _0, _1, _2
    )]
    ClusterDisplayNameDoesNotMatch(String, String, String),

    #[fail(
        display = "expected cluster ID '{}' but found '{}' for node with ID '{}'",
        _0, _1, _2
    )]
    ClusterIdDoesNotMatch(String, String, String),

    #[fail(display = "error emitting {} event", _0)]
    EventEmit(&'static str),

    #[fail(display = "error fetching {} from the store", _0)]
    StoreRead(&'static str),

    #[fail(display = "error persisting {} to the store", _0)]
    StoreWrite(&'static str),
}

impl ErrorKind {
    /// Returns true if the error was specific to an agent and not the core system.
    ///
    /// For example: a connection error or an invalid response are agent specific errors
    /// while database or events stream errors are about the platform.
    pub fn is_agent(&self) -> bool {
        match self {
            ErrorKind::AgentConnect(_) => true,
            ErrorKind::AgentRead(_, _) => true,
            ErrorKind::ClusterDisplayNameDoesNotMatch(_, _, _) => true,
            ErrorKind::ClusterIdDoesNotMatch(_, _, _) => true,
            ErrorKind::EventEmit(_) => false,
            ErrorKind::StoreRead(_) => false,
            ErrorKind::StoreWrite(_) => false,
        }
    }

    fn kind_name(&self) -> Option<&str> {
        let name = match self {
            ErrorKind::AgentConnect(_) => "AgentConnect",
            ErrorKind::AgentRead(_, _) => "AgentRead",
            ErrorKind::ClusterDisplayNameDoesNotMatch(_, _, _) => "ClusterDisplayNameDoesNotMatch",
            ErrorKind::ClusterIdDoesNotMatch(_, _, _) => "ClusterIdDoesNotMatch",
            ErrorKind::EventEmit(_) => "EventEmit",
            ErrorKind::StoreRead(_) => "StoreRead",
            ErrorKind::StoreWrite(_) => "StoreWrite",
        };
        Some(name)
    }
}

/// Short form alias for functions returning `Error`s.
pub type Result<T> = ::std::result::Result<T, Error>;
