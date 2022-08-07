use std::fmt;

use failure::Backtrace;
use failure::Context;
use failure::Fail;

use replicante_models_core::cluster::OrchestrateReportOutcome;

/// Dumb wrapper to carry `anyhow::Error`s as `failure::Fail`s.
pub struct AnyWrap(anyhow::Error);

impl From<anyhow::Error> for AnyWrap {
    fn from(error: anyhow::Error) -> AnyWrap {
        AnyWrap(error)
    }
}

impl Fail for AnyWrap {
    fn cause(&self) -> Option<&dyn Fail> {
        None
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        None
    }

    fn name(&self) -> Option<&str> {
        Some("AnyWrap")
    }
}

impl fmt::Display for AnyWrap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl fmt::Debug for AnyWrap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

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
    #[fail(display = "unable to orchestrate cluster {}.{}", _0, _1)]
    Orchestrate(String, String),

    #[fail(
        display = "another orchestration task is in progress for {}.{}",
        _0, _1
    )]
    ConcurrentOrchestrate(String, String),

    #[fail(display = "unable to deserialize task payload")]
    DeserializePayload,

    #[fail(display = "unable to fetch cluster settings record for {}.{}", _0, _1)]
    FetchSettings(String, String),

    #[fail(display = "unable to release orchestration lock for {}.{}", _0, _1)]
    ReleaseLock(String, String),

    #[fail(display = "no cluster settings record available for {}.{}", _0, _1)]
    SettingsNotFound(String, String),
}

impl ErrorKind {
    pub fn orchestrate<S1, S2>(namespace: S1, cluster_id: S2) -> ErrorKind
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        ErrorKind::Orchestrate(namespace.into(), cluster_id.into())
    }

    pub fn concurrent_orchestrate<S1, S2>(namespace: S1, cluster_id: S2) -> ErrorKind
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        ErrorKind::ConcurrentOrchestrate(namespace.into(), cluster_id.into())
    }

    pub fn fetch_settings<S1, S2>(namespace: S1, cluster_id: S2) -> ErrorKind
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        ErrorKind::FetchSettings(namespace.into(), cluster_id.into())
    }

    pub fn release_lock<S1, S2>(namespace: S1, cluster_id: S2) -> ErrorKind
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        ErrorKind::ReleaseLock(namespace.into(), cluster_id.into())
    }

    pub fn settings_not_found<S1, S2>(namespace: S1, cluster_id: S2) -> ErrorKind
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        ErrorKind::SettingsNotFound(namespace.into(), cluster_id.into())
    }

    fn kind_name(&self) -> Option<&str> {
        let name = match self {
            ErrorKind::Orchestrate(_, _) => "Orchestrate",
            ErrorKind::ConcurrentOrchestrate(_, _) => "ConcurrentOrchestrate",
            ErrorKind::DeserializePayload => "DeserializePayload",
            ErrorKind::FetchSettings(_, _) => "FetchSettings",
            ErrorKind::ReleaseLock(_, _) => "ReleaseLock",
            ErrorKind::SettingsNotFound(_, _) => "SettingsNotFound",
        };
        Some(name)
    }
}

/// Short form alias for functions returning `Error`s.
pub type Result<T> = ::std::result::Result<T, Error>;

/// Covert a result into an orchestrate report outcome.
pub fn orchestrate_outcome(result: &Result<()>) -> OrchestrateReportOutcome {
    if let Err(error) = result {
        let causes = <dyn Fail>::iter_causes(error);
        OrchestrateReportOutcome {
            error: Some(error.to_string()),
            error_causes: causes.skip(1).map(ToString::to_string).collect(),
            success: false,
        }
    } else {
        OrchestrateReportOutcome {
            error: None,
            error_causes: Vec::new(),
            success: true,
        }
    }
}
