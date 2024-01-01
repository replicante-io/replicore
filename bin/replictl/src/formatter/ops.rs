//! Operations offered by a `replictl` formatter interface.
use replisdk::core::models::namespace::Namespace;

use self::sealed::SealFormatOp;
use crate::context::Context;

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

    /// Request a strategy to format `NamespaceEntry` lists.
    NamespaceList,

    /// Format information about a [`Namespace`].
    Namespace(Namespace),
}

/// All known responses from format operations.
pub enum Responses {
    /// Return a object to format a list of [`Context`]s.
    ContextList(Box<dyn super::ContextList>),

    /// Return a object to format a list of `NamespaceEntry`s.
    NamespaceList(Box<dyn super::NamespaceList>),

    /// The formatting operation was successful.
    Success,
}

impl Responses {
    /// Wrap a [`ContextList`](super::ContextList) returned by the formatter.
    pub fn contexts<L>(value: L) -> Self
    where
        L: super::ContextList + 'static,
    {
        let value = Box::new(value);
        Self::ContextList(value)
    }

    /// Wrap a [`NamespaceList`](super::NamespaceList) returned by the formatter.
    pub fn namespaces<L>(value: L) -> Self
    where
        L: super::NamespaceList + 'static,
    {
        let value = Box::new(value);
        Self::NamespaceList(value)
    }
}

// --- Operation & return types -- //
/// Request a formatter to emit [`Context`] lists.
pub struct ContextListOp;

/// Request a formatter to emit `NamespaceEntry` lists.
pub struct NamespaceListOp;

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

impl SealFormatOp for Namespace {}
impl From<Namespace> for Ops {
    fn from(value: Namespace) -> Self {
        Self::Namespace(value)
    }
}
impl FormatOp for Namespace {
    type Response = ();
}

impl SealFormatOp for NamespaceListOp {}
impl From<NamespaceListOp> for Ops {
    fn from(_: NamespaceListOp) -> Self {
        Self::NamespaceList
    }
}
impl FormatOp for NamespaceListOp {
    type Response = Box<dyn super::NamespaceList>;
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
impl From<Responses> for Box<dyn super::NamespaceList> {
    fn from(value: Responses) -> Self {
        match value {
            Responses::NamespaceList(value) => value,
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
