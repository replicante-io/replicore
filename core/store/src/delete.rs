//! RepliCore Control Plane persistent store operations to delete records.
use replisdk::core::models::namespace::Namespace;

use self::seal::SealDeleteOp;

/// Internal trait to enable delete operations on the persistent store.
pub trait DeleteOp: Into<DeleteOps> + SealDeleteOp {
    /// Type returned by the matching delete operation.
    type Response: From<DeleteResponses>;
}

/// List of all delete operations the persistent store must implement.
pub enum DeleteOps {
    /// Delete a namespace by Namespace ID.
    Namespace(DeleteNamespace),
}

/// List of all responses from delete operations.
pub enum DeleteResponses {
    /// The operation completed successfully and does not return data.
    Success,
}

// --- High level delete operations --- //
/// Request deletion of a [`Namespace`] record.
pub struct DeleteNamespace {
    // Identifier of the [`Namespace`] record to delete.
    pub id: String,
}
impl From<&Namespace> for DeleteNamespace {
    fn from(value: &Namespace) -> Self {
        DeleteNamespace {
            id: value.id.clone(),
        }
    }
}
impl From<String> for DeleteNamespace {
    fn from(value: String) -> Self {
        DeleteNamespace { id: value }
    }
}
impl From<&str> for DeleteNamespace {
    fn from(value: &str) -> Self {
        DeleteNamespace {
            id: value.to_string(),
        }
    }
}

// --- Create internal implementation details follow --- //
/// Private module to seal implementation details.
mod seal {
    /// Super-trait to seal the [`DeleteOp`](super::DeleteOp) trait.
    pub trait SealDeleteOp {}
}

// --- Implement DeleteOp and super traits on types for transparent operations --- //
impl DeleteOp for DeleteNamespace {
    type Response = ();
}
impl SealDeleteOp for DeleteNamespace {}
impl From<DeleteNamespace> for DeleteOps {
    fn from(value: DeleteNamespace) -> Self {
        DeleteOps::Namespace(value)
    }
}

impl DeleteOp for &Namespace {
    type Response = ();
}
impl SealDeleteOp for &Namespace {}
impl From<&Namespace> for DeleteOps {
    fn from(value: &Namespace) -> Self {
        let value = DeleteNamespace::from(value);
        DeleteOps::Namespace(value)
    }
}

// --- Implement DeleteResponses conversions on return types for transparent operations --- //
impl From<DeleteResponses> for () {
    fn from(value: DeleteResponses) -> Self {
        match value {
            DeleteResponses::Success => (),
            //_ => panic!(TODO),
        }
    }
}
