use std::fmt;

use failure::Backtrace;
use failure::Context;
use failure::Fail;


/// Error information returned by the `Coordinator` API in case of errors.
#[derive(Debug)]
pub struct Error(Context<ErrorKind>);

impl Error {
    pub fn kind(&self) -> ErrorKind {
        *self.0.get_context()
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
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, Fail)]
pub enum ErrorKind {
    // TODO
    #[fail(display = "TODO: create errors")]
    __TODO__
}


/// Short form alias for functions returning `Error`s.
pub type Result<T> = ::std::result::Result<T, Error>;
