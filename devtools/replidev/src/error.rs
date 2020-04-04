use std::fmt;

use failure::Backtrace;
use failure::Context;
use failure::Fail;

use crate::conf::Project;

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

/// Exhaustive list of possible errors emitted by this crate.
#[derive(Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "failed to start {} command", _0)]
    CommandExec(String),

    #[fail(display = "{} command was not successful", _0)]
    CommandFailed(String),

    #[fail(
        display = "could not load configuration, are you in the root of a Replicante repository?"
    )]
    ConfigLoad,

    #[fail(display = "filesystem error: {}", _0)]
    FsError(String),

    #[fail(display = "{} command does not support the {} project", _0, _1)]
    InvalidProject(&'static str, Project),

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

    pub fn invalid_pod<S: Into<String>>(pod: S) -> Self {
        Self::PodNotValid(pod.into())
    }

    pub fn invalid_project(project: Project, command: &'static str) -> Self {
        Self::InvalidProject(command, project)
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
