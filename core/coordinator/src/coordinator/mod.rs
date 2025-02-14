//! Coordinated logic abstraction built on top of lease.
use anyhow::Result;

use replicore_context::Context;

use crate::State;

mod set;

pub use self::set::Coordinator;
pub use self::set::CoordinatorBuilder;

/// Interface for coordinated logic implementations.
///
/// When the process has exclusive ownership of the coordination [`Lease`](super::Lease)
/// the [`ICoordinated::primary`] async function is executed (and polled).
///
/// When the coordination lease move to [`State::Secondary`] the [`ICoordinated::secondary`]
/// async function is instead executed (and polled) instead.
///
/// For any [`State`] transition of the coordination lease the [`ICoordinated::transition`]
/// async function is executed with the relevant states.
#[async_trait::async_trait]
pub trait ICoordinated: Send + Sync {
    /// Execute the cluster exclusive logic in this method.
    ///
    /// The method is expected to continue running forever.
    /// When the coordination lease state changes the method's future is dropped
    /// and the [`ICoordinated::transition`] method is called.
    async fn primary(&self, context: &Context) -> Result<()>;

    /// Execute logic when the process does NOT hold the coordination [`Lease`](super::Lease).
    ///
    /// The method is expected to continue running forever.
    /// When the coordination lease state changes the method's future is dropped
    /// and the [`ICoordinated::transition`] method is called.
    ///
    /// In the default implementation the method runs forever without doing anything.
    async fn secondary(&self, context: &Context) -> Result<()>;

    /// Handle coordination lease [`State`] transition notifications.
    ///
    /// In the default implementation nothing happens.
    #[allow(unused_variables)]
    async fn transition(&self, context: &Context, from: State, to: State) -> Result<()> {
        Ok(())
    }
}
