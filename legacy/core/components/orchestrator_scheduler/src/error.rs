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
    #[fail(display = "unable to iterate over clusters to orchestrate search")]
    ClustersPartialSearch,

    #[fail(display = "unable to search for clusters to orchestrate")]
    ClustersSearch,

    #[fail(
        display = "unable to persist next_orchestrate update for {}.{}",
        _0, _1
    )]
    PersistNextOrchestrate(String, String),

    #[fail(display = "failed to spawn descovery thread")]
    ThreadSpawn,
}

impl ErrorKind {
    pub fn persist_next_orchestrate<S1, S2>(namespace: S1, cluster_id: S2) -> ErrorKind
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        ErrorKind::PersistNextOrchestrate(namespace.into(), cluster_id.into())
    }

    fn kind_name(&self) -> Option<&str> {
        let name = match self {
            ErrorKind::ClustersPartialSearch => "ClustersPartialSearch",
            ErrorKind::ClustersSearch => "ClustersSearch",
            ErrorKind::PersistNextOrchestrate(_, _) => "PersistNextOrchestrate",
            ErrorKind::ThreadSpawn => "ThreadSpawn",
        };
        Some(name)
    }
}

/// Short form alias for functions returning `Error`s.
pub type Result<T> = ::std::result::Result<T, Error>;
