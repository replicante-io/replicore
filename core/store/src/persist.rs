//! RepliCore Control Plane persistent store operations to persist records.
use replisdk::core::models::namespace::Namespace;

use self::seal::SealPersistOp;

/// Internal trait to enable persist operations on the persistent store.
pub trait PersistOp: Into<PersistOps> + SealPersistOp {
    /// Type returned by the matching persist operation.
    type Response: From<PersistResponses>;
}

/// List of all persist operations the persistent store must implement.
pub enum PersistOps {
    /// Persist a namespace record.
    Namespace(Namespace),
}

/// List of all responses from persist operations.
pub enum PersistResponses {
    /// The operation completed successfully and does not return data.
    Success,
}

// --- High level query operations --- //
// TODO: define as needed or remove if none after feature parity.

// --- Create internal implementation details follow --- //
/// Private module to seal implementation details.
mod seal {
    /// Super-trait to seal the [`PersistOp`](super::PersistOp) trait.
    pub trait SealPersistOp {}
}

// --- Implement PersistOp and super traits on types for transparent operations --- //
impl PersistOp for Namespace {
    type Response = ();
}
impl SealPersistOp for Namespace {}
impl From<Namespace> for PersistOps {
    fn from(value: Namespace) -> Self {
        PersistOps::Namespace(value)
    }
}

// --- Implement PersistResponses conversions on return types for transparent operations --- //
impl From<PersistResponses> for () {
    fn from(value: PersistResponses) -> Self {
        match value {
            PersistResponses::Success => (),
            //_ => panic!(TODO),
        }
    }
}
