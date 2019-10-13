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
    #[fail(display = "unable to acknowledge message")]
    AckFailed,

    #[fail(display = "unable to create a client to the backend")]
    BackendClientCreation,

    #[fail(display = "unable to emit message")]
    EmitFailed,

    #[fail(display = "unable to follow stream")]
    FollowFailed,

    #[fail(display = "unable to decode value for header '{}'", _0)]
    MessageInvalidHeader(String),

    #[fail(display = "unable to decode message without a payload")]
    MessageNoPayload,

    #[fail(display = "unable to encode message payload")]
    PayloadEncode,

    #[fail(display = "unable to decode message payload")]
    PayloadDecode,
}

impl ErrorKind {
    fn kind_name(&self) -> Option<&str> {
        let name = match self {
            ErrorKind::AckFailed => "AckFailed",
            ErrorKind::BackendClientCreation => "BackendClientCreation",
            ErrorKind::EmitFailed => "EmitFailed",
            ErrorKind::FollowFailed => "FollowFailed",
            ErrorKind::MessageInvalidHeader(_) => "MessageInvalidHeader",
            ErrorKind::MessageNoPayload => "MessageNoPayload",
            ErrorKind::PayloadEncode => "PayloadEncode",
            ErrorKind::PayloadDecode => "PayloadDecode",
        };
        Some(name)
    }
}

/// Short form alias for functions returning `Error`s.
pub type Result<T> = ::std::result::Result<T, Error>;
