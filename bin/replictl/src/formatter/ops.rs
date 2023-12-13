//! Operations offered by a `replictl` formatter interface.
use crate::context::Context;

use self::sealed::SealFormatOp;

/// Internal trait to support ergonomic formatting operations.
pub trait FormatOp: Into<Ops> + SealFormatOp {
    /// Type returned by the matching format operation.
    type Response: From<Responses>;
}

/// All known operations that must be implemented by formatters.
pub enum Ops {
    /// Format information about a [`Context`].
    Context(Context),

    /// Request a formatter to emit [`Context`] lists.
    ContextList,
}

/// All known responses from format operations.
pub enum Responses {
    /// Return a object to format a list of [`Context`]s.
    ContextList(Box<dyn super::ContextList>),

    /// The formatting operation was successful.
    Success,
}

impl<L> From<L> for Responses
where
    L: super::ContextList + 'static,
{
    fn from(value: L) -> Self {
        let value = Box::new(value);
        Self::ContextList(value)
    }
}

// --- Operation & return types -- //
/// Request a formatter to emit [`Context`] lists.
pub struct ContextListOp;

/// Private module to seal implementation details.
mod sealed {
    /// Super-trait to seal the [`FormatOp`](super::FormatOp) trait.
    pub trait SealFormatOp {}
}

// --- Implement FormatOp and other traits on types for transparent operations --- //
impl SealFormatOp for Context {}
impl From<Context> for Ops {
    fn from(value: Context) -> Self {
        Self::Context(value)
    }
}
impl FormatOp for Context {
    type Response = ();
}

impl SealFormatOp for ContextListOp {}
impl From<ContextListOp> for Ops {
    fn from(_: ContextListOp) -> Self {
        Self::ContextList
    }
}
impl FormatOp for ContextListOp {
    type Response = Box<dyn super::ContextList>;
}

// --- Implement Responses conversions on return types for transparent operations --- //
impl From<Responses> for Box<dyn super::ContextList> {
    fn from(value: Responses) -> Self {
        match value {
            Responses::ContextList(value) => value,
            _ => panic!("unexpected response type for formatter operation"),
        }
    }
}
impl From<Responses> for () {
    fn from(value: Responses) -> Self {
        match value {
            Responses::Success => (),
            _ => panic!("unexpected response type for formatter operation"),
        }
    }
}
