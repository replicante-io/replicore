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

    /// True if the error was caused by missing information on the agent.
    pub fn not_found(&self) -> bool {
        matches!(self.kind(), ErrorKind::NotFound(_, _))
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
    fn cause(&self) -> Option<&dyn Fail> {
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
    #[fail(display = "attempted to schedule an action that was already scheduled")]
    DuplicateAction,

    #[fail(display = "unable to decode JSON data")]
    JsonDecode,

    #[fail(display = "no {} with id '{}' found", _0, _1)]
    NotFound(&'static str, String),

    #[fail(display = "remote error: {}", _0)]
    Remote(String),

    #[fail(display = "{} transport error", _0)]
    Transport(&'static str),
}

impl ErrorKind {
    pub fn is_duplicate_action(&self) -> bool {
        matches!(self, ErrorKind::DuplicateAction)
    }

    fn kind_name(&self) -> Option<&str> {
        let name = match self {
            ErrorKind::DuplicateAction => "DuplicateAction",
            ErrorKind::JsonDecode => "JsonDecode",
            ErrorKind::NotFound(_, _) => "NotFound",
            ErrorKind::Remote(_) => "Remote",
            ErrorKind::Transport(_) => "Transport",
        };
        Some(name)
    }
}

/// Short form alias for functions returning `Error`s.
pub type Result<T> = ::std::result::Result<T, Error>;
