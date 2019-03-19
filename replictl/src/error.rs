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

impl From<::failure::Error> for Error {
    fn from(error: ::failure::Error) -> Error {
        ErrorKind::Legacy(error).into()
    }
}


/// Exhaustive list of possible errors emitted by this crate.
#[derive(Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "could not initialise admin interface for {}", _0)]
    AdminInit(&'static str),

    #[fail(display = "could not check {}", _0)]
    CheckFailed(&'static str),

    #[fail(display = "could not initialise client interface for {}", _0)]
    ClientInit(&'static str),

    #[fail(display = "invalid configuration: {}", _0)]
    Config(&'static str),

    #[fail(display = "could not fetch {} version", _0)]
    FetchVersion(&'static str),

    #[fail(display = "could not instantiate HTTP client")]
    HttpClient,

    #[fail(display = "I/O error on file {}", _0)]
    Io(String),

    #[fail(display = "need a command to run for '{}'", _0)]
    NoCommand(&'static str),

    #[fail(display = "could not JSON decode API response from replicante")]
    ReplicanteJsonDecode,

    #[fail(display = "replicante API request to '{}' failed", _0)]
    ReplicanteRequest(&'static str),

    #[fail(display = "unkown '{}' command for '{}'", _1, _0)]
    UnkownSubcommand(&'static str, String),

    // TODO: drop once all uses are removed.
    #[fail(display = "{}", _0)]
    #[deprecated(since = "0.2.0", note = "move to specific ErrorKinds")]
    Legacy(#[cause] ::failure::Error),
}


/// Short form alias for functions returning `Error`s.
pub type Result<T> = ::std::result::Result<T, Error>;
