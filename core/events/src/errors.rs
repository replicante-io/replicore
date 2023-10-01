//! Errors returned by the replicore-events crate.

/// Errors dealing with events.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Unable to decode event payload into the the specified type.
    #[error("unable to decode event payload into the the specified type")]
    PayloadDecode,
}
