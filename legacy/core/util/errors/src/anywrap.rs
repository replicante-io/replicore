use std::fmt;

use failure::Backtrace;
use failure::Fail;

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
