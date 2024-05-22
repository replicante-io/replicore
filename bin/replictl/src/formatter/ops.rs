//! Operations offered by a `replictl` formatter interface.
use anyhow::Error;
use anyhow::Result;

use replisdk::core::models::cluster::ClusterDiscovery;
use replisdk::core::models::cluster::ClusterSpec;
use replisdk::core::models::namespace::Namespace;
use replisdk::core::models::oaction::OAction;
use replisdk::core::models::platform::Platform;

use self::sealed::SealFormatOp;
use crate::context::Context;

/// Internal trait to support ergonomic formatting operations.
pub trait FormatOp: Into<Ops> + SealFormatOp {
    /// Type returned by the matching format operation.
    type Response: From<Responses>;
}

/// All known operations that must be implemented by formatters.
pub enum Ops {
    /// Format information about a [`ClusterDiscovery`].
    ClusterDiscovery(ClusterDiscovery),

    /// Format information about a [`ClusterSpec`].
    ClusterSpec(ClusterSpec),

    /// Request a strategy to format `ClusterSpecEntry` lists.
    ClusterSpecList,

    /// Format information about a [`Context`].
    Context(Context),

    /// Request a formatter to emit [`Context`] lists.
    ContextList,

    /// Request a strategy to format `NamespaceEntry` lists.
    NamespaceList,

    /// Format information about a [`Namespace`].
    Namespace(Namespace),

    /// Format information about an [`OAction`].
    OAction(OAction),

    /// Request a strategy to format `OActionEntry` lists.
    OActionList,

    /// Request a strategy to format `PlatformEntry` lists.
    PlatformList,

    /// Format information about a [`Platform`].
    Platform(Platform),
}

/// All known responses from format operations.
pub enum Responses {
    /// Return a object to format a list of `ClusterSpecEntry`s.
    ClusterSpecList(Box<dyn super::ClusterSpecList>),

    /// Return a object to format a list of [`Context`]s.
    ContextList(Box<dyn super::ContextList>),

    /// Return an error back to the caller.
    Err(Error),

    /// Return a object to format a list of `NamespaceEntry`s.
    NamespaceList(Box<dyn super::NamespaceList>),

    /// Return a object to format a list of `OActionEntry`s.
    OActionList(Box<dyn super::OActionList>),

    /// Return a object to format a list of `PlatformEntry`s.
    PlatformList(Box<dyn super::PlatformList>),

    /// The formatting operation was successful.
    Success,
}

impl Responses {
    /// Wrap a [`ClusterSpecList`](super::ClusterSpecList) returned by the formatter.
    pub fn cluster_specs<L>(value: L) -> Self
    where
        L: super::ClusterSpecList + 'static,
    {
        let value = Box::new(value);
        Self::ClusterSpecList(value)
    }

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

    /// Wrap an [`OActionList`](super::OActionList) returned by the formatter.
    pub fn oactions<L>(value: L) -> Self
    where
        L: super::OActionList + 'static,
    {
        let value = Box::new(value);
        Self::OActionList(value)
    }

    /// Wrap a [`PlatformList`](super::PlatformList) returned by the formatter.
    pub fn platforms<L>(value: L) -> Self
    where
        L: super::PlatformList + 'static,
    {
        let value = Box::new(value);
        Self::PlatformList(value)
    }
}

// --- Operation & return types -- //
/// Request a formatter to emit `ClusterSpecEntry` lists.
pub struct ClusterSpecListOp;

/// Request a formatter to emit [`Context`] lists.
pub struct ContextListOp;

/// Request a formatter to emit `NamespaceEntry` lists.
pub struct NamespaceListOp;

/// Request a formatter to emit `OActionEntry` lists.
pub struct OActionListOp;

/// Request a formatter to emit `PlatformEntry` lists.
pub struct PlatformListOp;

/// Private module to seal implementation details.
mod sealed {
    /// Super-trait to seal the [`FormatOp`](super::FormatOp) trait.
    pub trait SealFormatOp {}
}

// --- Implement FormatOp and other traits on types for transparent operations --- //
impl SealFormatOp for ClusterDiscovery {}
impl From<ClusterDiscovery> for Ops {
    fn from(value: ClusterDiscovery) -> Self {
        Self::ClusterDiscovery(value)
    }
}
impl FormatOp for ClusterDiscovery {
    type Response = ();
}

impl SealFormatOp for ClusterSpec {}
impl From<ClusterSpec> for Ops {
    fn from(value: ClusterSpec) -> Self {
        Self::ClusterSpec(value)
    }
}
impl FormatOp for ClusterSpec {
    type Response = ();
}

impl SealFormatOp for ClusterSpecListOp {}
impl From<ClusterSpecListOp> for Ops {
    fn from(_: ClusterSpecListOp) -> Self {
        Self::ClusterSpecList
    }
}
impl FormatOp for ClusterSpecListOp {
    type Response = Box<dyn super::ClusterSpecList>;
}

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

impl SealFormatOp for OAction {}
impl From<OAction> for Ops {
    fn from(value: OAction) -> Self {
        Self::OAction(value)
    }
}
impl FormatOp for OAction {
    type Response = Result<()>;
}

impl SealFormatOp for OActionListOp {}
impl From<OActionListOp> for Ops {
    fn from(_: OActionListOp) -> Self {
        Self::OActionList
    }
}
impl FormatOp for OActionListOp {
    type Response = Box<dyn super::OActionList>;
}

impl SealFormatOp for Platform {}
impl From<Platform> for Ops {
    fn from(value: Platform) -> Self {
        Self::Platform(value)
    }
}
impl FormatOp for Platform {
    type Response = ();
}

impl SealFormatOp for PlatformListOp {}
impl From<PlatformListOp> for Ops {
    fn from(_: PlatformListOp) -> Self {
        Self::PlatformList
    }
}
impl FormatOp for PlatformListOp {
    type Response = Box<dyn super::PlatformList>;
}

// --- Implement Responses conversions on return types for transparent operations --- //
impl From<Responses> for Box<dyn super::ClusterSpecList> {
    fn from(value: Responses) -> Self {
        match value {
            Responses::ClusterSpecList(value) => value,
            _ => panic!("unexpected response type for formatter operation"),
        }
    }
}
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
impl From<Responses> for Box<dyn super::OActionList> {
    fn from(value: Responses) -> Self {
        match value {
            Responses::OActionList(value) => value,
            _ => panic!("unexpected response type for formatter operation"),
        }
    }
}
impl From<Responses> for Box<dyn super::PlatformList> {
    fn from(value: Responses) -> Self {
        match value {
            Responses::PlatformList(value) => value,
            _ => panic!("unexpected response type for formatter operation"),
        }
    }
}
impl From<Responses> for Result<()> {
    fn from(value: Responses) -> Self {
        match value {
            Responses::Err(error) => Err(error),
            Responses::Success => Ok(()),
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
