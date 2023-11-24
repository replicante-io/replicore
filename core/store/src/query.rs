//! RepliCore Control Plane persistent store operations to query records.
use anyhow::Result;
use futures::Stream;

use replisdk::core::models::namespace::Namespace;
use replisdk::core::models::platform::Platform;

use self::seal::SealQueryOp;
use crate::ids::NamespaceID;
use crate::ids::NamespacedResourceID;

/// Internal trait to enable query operations on the persistent store.
pub trait QueryOp: Into<QueryOps> + SealQueryOp {
    /// Type returned by the matching query operation.
    type Response: From<QueryResponses>;
}

/// List of all query operations the persistent store must implement.
pub enum QueryOps {
    /// List the IDs of all known namespaces, sorted alphabetically.
    ListNamespaceIds,

    /// List the Ids of all known platforms in the namespace, sorted alphabetically.
    ListPlatformIds(NamespaceID),

    /// Query a namespace by Namespace ID.
    Namespace(LookupNamespace),

    /// Query a platform by Namespace ID and Resource Name.
    Platform(NamespacedResourceID),
}

/// List of all responses from query operations.
pub enum QueryResponses {
    /// Return a [`Namespace`], if one was found matching the query.
    Namespace(Option<Namespace>),

    /// Return a [`Platform`], if one was found matching the query.
    Platform(Option<Platform>),

    /// Return a [`Stream`] (async iterator) of strings (useful for IDs).
    StringStream(StringStream),
}

// --- Operations return types -- //
/// Alias for a heap-allocated [`Stream`] of strings (useful for IDs).
pub type StringStream = std::pin::Pin<Box<dyn Stream<Item = Result<String>>>>;

// --- High level query operations --- //
/// List the IDs of all known namespaces, sorted alphabetically.
pub struct ListNamespaceIds;

/// List the IDs of all known platforms in a namespace, sorted alphabetically.
pub struct ListPlatformIds(pub NamespaceID);

/// Lookup a [`Namespace`] record by ID.
#[derive(Clone, Debug)]
pub struct LookupNamespace(pub NamespaceID);
impl From<&Namespace> for LookupNamespace {
    fn from(value: &Namespace) -> Self {
        let id = value.id.clone();
        let value = NamespaceID { id };
        LookupNamespace(value)
    }
}
impl From<String> for LookupNamespace {
    fn from(value: String) -> Self {
        let value = NamespaceID { id: value };
        LookupNamespace(value)
    }
}
impl From<&str> for LookupNamespace {
    fn from(value: &str) -> Self {
        let id = value.to_string();
        let value = NamespaceID { id };
        LookupNamespace(value)
    }
}

/// Lookup a [`Platform`] record by ID.
#[derive(Clone, Debug)]
pub struct LookupPlatform(pub NamespacedResourceID);
impl From<&Platform> for LookupPlatform {
    fn from(value: &Platform) -> Self {
        let name = value.name.clone();
        let ns_id = value.ns_id.clone();
        let value = NamespacedResourceID { name, ns_id };
        LookupPlatform(value)
    }
}

// --- Create internal implementation details follow --- //
/// Private module to seal implementation details.
mod seal {
    /// Super-trait to seal the [`QueryOp`](super::QueryOp) trait.
    pub trait SealQueryOp {}
}

// --- Implement QueryOp and super traits on types for transparent operations --- //
impl SealQueryOp for ListNamespaceIds {}
impl QueryOp for ListNamespaceIds {
    type Response = StringStream;
}
impl From<ListNamespaceIds> for QueryOps {
    fn from(_: ListNamespaceIds) -> Self {
        QueryOps::ListNamespaceIds
    }
}

impl SealQueryOp for ListPlatformIds {}
impl QueryOp for ListPlatformIds {
    type Response = StringStream;
}
impl From<ListPlatformIds> for QueryOps {
    fn from(value: ListPlatformIds) -> Self {
        QueryOps::ListPlatformIds(value.0)
    }
}

impl SealQueryOp for LookupNamespace {}
impl QueryOp for LookupNamespace {
    type Response = Option<Namespace>;
}
impl From<LookupNamespace> for QueryOps {
    fn from(value: LookupNamespace) -> Self {
        QueryOps::Namespace(value)
    }
}

impl SealQueryOp for LookupPlatform {}
impl QueryOp for LookupPlatform {
    type Response = Option<Platform>;
}
impl From<LookupPlatform> for QueryOps {
    fn from(value: LookupPlatform) -> Self {
        QueryOps::Platform(value.0)
    }
}

// --- Implement QueryResponses conversions on return types for transparent operations --- //
impl From<QueryResponses> for Option<Namespace> {
    fn from(value: QueryResponses) -> Self {
        match value {
            QueryResponses::Namespace(namespace) => namespace,
            _ => panic!("unexpected result type for the given query operation"),
        }
    }
}
impl From<QueryResponses> for Option<Platform> {
    fn from(value: QueryResponses) -> Self {
        match value {
            QueryResponses::Platform(platform) => platform,
            _ => panic!("unexpected result type for the given query operation"),
        }
    }
}
impl From<QueryResponses> for StringStream {
    fn from(value: QueryResponses) -> Self {
        match value {
            QueryResponses::StringStream(stream) => stream,
            _ => panic!("unexpected result type for the given query operation"),
        }
    }
}
