//! RepliCore Control Plane persistent store operations to query records.
use replisdk::core::models::namespace::Namespace;

use self::seal::SealQueryOp;

/// Internal trait to enable query operations on the persistent store.
pub trait QueryOp: Into<QueryOps> + SealQueryOp {
    /// Type returned by the matching query operation.
    type Response: From<QueryResponses>;
}

/// List of all query operations the persistent store must implement.
pub enum QueryOps {
    /// Query a namespace by Namespace ID.
    Namespace(LookupNamespace),
}

/// List of all responses from query operations.
pub enum QueryResponses {
    /// Return a namespace, if one was found matching the query.
    Namespace(Option<Namespace>),
}

// --- High level query operations --- //
/// Lookup a [`Namespace`] record by ID.
#[derive(Clone, Debug)]
pub struct LookupNamespace {
    // Identifier of the [`Namespace`] record to lookup.
    pub id: String,
}
impl From<Namespace> for LookupNamespace {
    fn from(value: Namespace) -> Self {
        LookupNamespace { id: value.id }
    }
}
impl From<&Namespace> for LookupNamespace {
    fn from(value: &Namespace) -> Self {
        LookupNamespace {
            id: value.id.clone(),
        }
    }
}
impl From<String> for LookupNamespace {
    fn from(value: String) -> Self {
        LookupNamespace { id: value }
    }
}
impl From<&str> for LookupNamespace {
    fn from(value: &str) -> Self {
        LookupNamespace {
            id: value.to_string(),
        }
    }
}

// --- Create internal implementation details follow --- //
/// Private module to seal implementation details.
mod seal {
    /// Super-trait to seal the [`QueryOp`](super::QueryOp) trait.
    pub trait SealQueryOp {}
}

// --- Implement QueryOp and super traits on types for transparent operations --- //
impl QueryOp for LookupNamespace {
    type Response = Option<Namespace>;
}
impl SealQueryOp for LookupNamespace {}
impl From<LookupNamespace> for QueryOps {
    fn from(value: LookupNamespace) -> Self {
        QueryOps::Namespace(value)
    }
}

// --- Implement QueryResponses conversions on return types for transparent operations --- //
impl From<QueryResponses> for Option<Namespace> {
    fn from(value: QueryResponses) -> Self {
        match value {
            QueryResponses::Namespace(namespace) => namespace,
            //_ => panic!(TODO),
        }
    }
}
