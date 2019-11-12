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

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> {
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
    #[fail(display = "CLI option --{} is required", _0)]
    CliOptMissing(&'static str),

    #[fail(display = "unable to open file '{}'", _0)]
    FsOpen(String),

    #[fail(display = "unable to create directory '{}'", _0)]
    FsMkDir(String),

    #[fail(display = "unable to expand ~ to the user's home")]
    HomeNotFound,

    #[fail(display = "need a command to run for '{}'", _0)]
    NoCommand(String),

    #[fail(display = "SSO session '{}' not available", _0)]
    SessionNotFound(String),

    #[fail(display = "unable to decode the sessions store")]
    SessionsDecode,

    #[fail(display = "unable to encode the sessions store")]
    SessionsEncode,

    #[fail(display = "unkown '{}' command for '{}'", _1, _0)]
    UnkownSubcommand(String, String),

    #[fail(display = "user interaction failed")]
    UserInteraction,
}

/// Short form alias for functions returning `Error`s.
pub type Result<T> = ::std::result::Result<T, Error>;
