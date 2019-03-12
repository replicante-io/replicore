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

impl From<::replicante_data_store::Error> for Error {
    fn from(error: ::replicante_data_store::Error) -> Error {
        ErrorKind::LegacyStore(error).into()
    }
}

impl From<::replicante_streams_events::Error> for Error {
    fn from(error: ::replicante_streams_events::Error) -> Error {
        ErrorKind::LegacyStreamEvent(error).into()
    }
}


/// Exhaustive list of possible errors emitted by this crate.
#[derive(Debug, Fail)]
pub enum ErrorKind {
    // TODO: drop once all uses are removed.
    #[fail(display = "{}", _0)]
    Legacy(#[cause] ::failure::Error),

    #[fail(display = "{}", _0)]
    #[deprecated(since = "0.1.0", note = "store was convered to failure")]
    LegacyStore(#[cause] ::replicante_data_store::Error),

    #[fail(display = "{}", _0)]
    #[deprecated(since = "0.1.0", note = "event stream was convered to failure")]
    LegacyStreamEvent(#[cause] ::replicante_streams_events::Error),
}


/// Short form alias for functions returning `Error`s.
pub type Result<T> = ::std::result::Result<T, Error>;
