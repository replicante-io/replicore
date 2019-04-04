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
    #[fail(display = "unable to create a client to the backend")]
    BackendClientCreation,

    #[fail(display = "unable to commit task as processed")]
    CommitFailed,

    #[fail(display = "stuck trying to commit replayed task with ID '{}'", _0)]
    CommitRetryStuck(String),

    #[fail(display = "unable to fetch tasks to work on")]
    FetchError,

    #[fail(display = "unable to deserialize task payload into the desired structure")]
    PayloadDeserialize,

    #[fail(display = "unable to serialize task into a payload to send")]
    PayloadSerialize,

    #[fail(display = "failed to sapwn some worker threads")]
    PoolSpawn,

    #[fail(display = "invalid queue name '{}'", _0)]
    QueueNameInvalid(String),

    #[fail(display = "unable to schedule retry of task")]
    RetryEnqueue,

    #[fail(display = "unable to schedule retry of task with ID '{}'", _0)]
    RetryEnqueueID(String),

    #[fail(display = "cannot {} tasks while scanning", _0)]
    ScanCannotAck(&'static str),

    #[fail(display = "invalid value '{}' for task header '{}'", _1, _0)]
    TaskHeaderInvalid(String, String),

    #[fail(display = "invalid task ID '{}'", _0)]
    TaskInvalidID(String),

    #[fail(display = "received task without ID")]
    TaskNoId,

    #[fail(display = "received task without payload (id: {})", _0)]
    TaskNoPayload(String),

    #[fail(display = "failed to request task")]
    TaskRequest,

    #[fail(display = "unable to subsribe for task delivery")]
    TaskSubscription,
}

/// Short form alias for functions returning `Error`s.
pub type Result<T> = ::std::result::Result<T, Error>;
