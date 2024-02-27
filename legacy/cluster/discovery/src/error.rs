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
    fn cause(&self) -> Option<&dyn Fail> {
        self.0.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.0.backtrace()
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

/// Exhaustive list of possible errors emitted by this crate.
#[derive(Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "unable to load PEM certificate for HTTP client")]
    HttpCertLoad,

    #[fail(display = "unable to initialise HTTP client")]
    HttpClient,

    #[fail(display = "invalid HTTP header name '{}'", _0)]
    HttpHeaderName(String),

    #[fail(display = "invalid HTTP header value '{}'", _0)]
    HttpHeaderValue(String),

    #[fail(display = "HTTP request failed")]
    HttpRequest,

    #[fail(display = "Invalid HTTP URL for platform")]
    HttpUrlInvalid,
}

impl ErrorKind {
    fn kind_name(&self) -> Option<&str> {
        let name = match self {
            ErrorKind::HttpCertLoad => "HttpCertLoad",
            ErrorKind::HttpClient => "HttpClient",
            ErrorKind::HttpHeaderName(_) => "HttpHeaderName",
            ErrorKind::HttpHeaderValue(_) => "HttpHeaderValue",
            ErrorKind::HttpRequest => "HttpRequest",
            ErrorKind::HttpUrlInvalid => "HttpUrlInvalid",
        };
        Some(name)
    }
}

/// Short form alias for functions returning `Error`s.
pub type Result<T> = ::std::result::Result<T, Error>;
