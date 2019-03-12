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
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}


/// Exhaustive list of possible errors emitted by this crate.
#[derive(Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "mock {} not yet implemented", _0)]
    MockNotYetImplemented(&'static str),

    #[fail(display = "failed to connect to MongoDB URI '{}'", _0)]
    MongoDBConnect(String),

    #[fail(display = "failed to read MongoDB cursor for {} operation", _0)]
    MongoDBCursor(&'static str),

    #[fail(display = "MongoDB BSON decode failed")]
    MongoDBBsonDecode,

    #[fail(display = "MongoDB BSON encode failed")]
    MongoDBBsonEncode,

    #[fail(display = "MongoDB {} operation failed", _0)]
    MongoDBOperation(&'static str),

    #[fail(display = "unable to parse model with id '{}': {}", _0, _1)]
    UnableToParseModel(String, String),
}


/// Short form alias for functions returning `Error`s.
pub type Result<T> = ::std::result::Result<T, Error>;
