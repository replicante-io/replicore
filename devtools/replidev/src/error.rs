use std::fmt;

use failure::Backtrace;
use failure::Context;
use failure::Fail;

use crate::conf::Project;

#[derive(thiserror::Error, Debug)]
#[error("The replidev {command} command does not support the {project} project")]
pub struct InvalidProject {
    command: &'static str,
    project: Project,
}

impl InvalidProject {
    pub fn new(project: Project, command: &'static str) -> InvalidProject {
        InvalidProject { command, project }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("The release check process raised some issues")]
pub struct ReleaseCheck {
    /// List of errors raised by the release check process.
    pub errors: Vec<anyhow::Error>,
}

impl ReleaseCheck {
    /// Check a result for `ReleaseCheck` and expand the collections of errors.
    ///
    /// If the result failed with a `ReleaseCheck` error then the errors in it are
    /// added to the errors in this instance and `Ok(())` is return.
    ///
    /// Any other error is returned unchanged.
    pub fn check(&mut self, result: anyhow::Result<()>) -> anyhow::Result<()> {
        let error = match result {
            Ok(()) => return Ok(()),
            Err(error) => error,
        };
        match error.downcast::<ReleaseCheck>() {
            Err(error) => Err(error),
            Ok(issues) => {
                self.errors.extend(issues.errors);
                Ok(())
            }
        }
    }

    /// Create a `ReleaseCheck` containing the given error.
    pub fn failed<E>(error: E) -> anyhow::Result<()>
    where
        E: Into<anyhow::Error>,
    {
        let errors = vec![error.into()];
        let error = anyhow::anyhow!(ReleaseCheck { errors });
        Err(error)
    }

    /// Convert this collection of errors into a result.
    ///
    /// If no errors were collected returns `Ok(())` otherwise returns itself as an error.
    pub fn into_result(self) -> anyhow::Result<()> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(self))
        }
    }

    pub fn new() -> ReleaseCheck {
        ReleaseCheck { errors: Vec::new() }
    }
}

impl From<anyhow::Error> for ReleaseCheck {
    fn from(error: anyhow::Error) -> ReleaseCheck {
        let errors = vec![error];
        ReleaseCheck { errors }
    }
}

// TODO: remove failure errors and code once replacement is complete.
/// Error information returned by functions in case of errors.
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

/// Temporary wrapper to implement `std::error::Error` for `Error` without conflicts with `Fail`.
pub fn wrap_for_anyhow(error: Error) -> anyhow::Error {
    error.compat().into()
}

/// Exhaustive list of possible errors emitted by this crate.
#[derive(Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "failed to start {} command", _0)]
    CommandExec(String),

    #[fail(display = "{} command was not successful", _0)]
    CommandFailed(String),

    #[fail(display = "filesystem error: {}", _0)]
    FsError(String),

    #[fail(display = "io error: {}", _0)]
    IoError(String),

    #[fail(display = "received invalid variable '{}': {}", _0, _1)]
    InvalidCliVar(String, String),

    #[fail(display = "received invalid variable JSON file '{}'", _0)]
    InvalidCliVarFile(String),

    #[fail(display = "hostname not set or not utf-8 in the HOSTNAME environment variable")]
    InvalidHostnameVar,

    #[fail(display = "could not find a non-loopback IP address")]
    IpNotDetected,

    #[fail(display = "invalid definition for the {} pod", _0)]
    PodNotValid(String),

    #[fail(display = "unable to find a defintion for the {} pod", _0)]
    PodNotFound(String),

    #[fail(display = "unable to decode response from {}", _0)]
    ResponseDecode(String),

    #[fail(display = "template rendering error")]
    TemplateRender,
}

impl ErrorKind {
    pub fn command_exec<S: std::fmt::Display>(cmd: S) -> Self {
        Self::CommandExec(cmd.to_string())
    }

    pub fn command_failed<S: std::fmt::Display>(cmd: S) -> Self {
        Self::CommandFailed(cmd.to_string())
    }

    pub fn fs_error(error: &str) -> Self {
        Self::FsError(error.to_string())
    }

    pub fn fs_not_allowed<S: std::fmt::Display>(path: S) -> Self {
        Self::FsError(format!("unable to access {}", path))
    }

    pub fn io(error: &str) -> Self {
        Self::IoError(error.to_string())
    }

    pub fn invalid_cli_var<N, R>(name: N, reason: R) -> Self
    where
        N: Into<String>,
        R: Into<String>,
    {
        Self::InvalidCliVar(name.into(), reason.into())
    }

    pub fn invalid_cli_var_file<N: Into<String>>(name: N) -> Self {
        Self::InvalidCliVarFile(name.into())
    }

    pub fn invalid_hostname_var() -> Self {
        Self::InvalidHostnameVar
    }

    pub fn invalid_pod<S: Into<String>>(pod: S) -> Self {
        Self::PodNotValid(pod.into())
    }

    pub fn ip_not_detected() -> Self {
        Self::IpNotDetected
    }

    pub fn pod_not_found<S: Into<String>>(pod: S) -> Self {
        Self::PodNotFound(pod.into())
    }

    pub fn podman_exec<S: std::fmt::Display>(cmd: S) -> Self {
        Self::CommandExec(format!("podman {}", cmd))
    }

    pub fn podman_failed<S: std::fmt::Display>(cmd: S) -> Self {
        Self::CommandFailed(format!("podman {}", cmd))
    }

    pub fn response_decode<S: Into<String>>(cmd: S) -> Self {
        Self::ResponseDecode(cmd.into())
    }

    pub fn template_render() -> Self {
        Self::TemplateRender
    }
}

/// Short form alias for functions returning `Error`s.
pub type Result<T> = ::std::result::Result<T, Error>;
