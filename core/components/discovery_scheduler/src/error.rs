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
    #[fail(display = "unable to iterate over discoveries search")]
    DiscoveriesPartialSearch,

    #[fail(display = "unable to search for discoveries to run")]
    DiscoveriesSearch,

    #[fail(display = "unable to persist next_run update for {}.{}", _0, _1)]
    PersistNextRun(String, String),

    #[fail(display = "failed to spawn descovery thread")]
    ThreadSpawn,
}

impl ErrorKind {
    pub fn persist_next_run<S1, S2>(namespace: S1, name: S2) -> ErrorKind
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        ErrorKind::PersistNextRun(namespace.into(), name.into())
    }

    fn kind_name(&self) -> Option<&str> {
        let name = match self {
            ErrorKind::DiscoveriesPartialSearch => "DiscoveriesPartialSearch",
            ErrorKind::DiscoveriesSearch => "DiscoveriesSearch",
            ErrorKind::PersistNextRun(_, _) => "PersistNextRun",
            ErrorKind::ThreadSpawn => "ThreadSpawn",
        };
        Some(name)
    }
}

/// Short form alias for functions returning `Error`s.
pub type Result<T> = ::std::result::Result<T, Error>;
