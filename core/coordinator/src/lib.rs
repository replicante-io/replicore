//! Replicante Control Plane distributed coordinator for exclusive logic execution.
//!
//! This service provides a framework for Replicante Core logic to ensure logic execution
//! happens only once at the same time in the cluster, regardless of how many nodes are involved.

mod coordinator;
mod factory;
mod lease;
mod lock;
mod time;

pub use self::coordinator::Coordinator;
pub use self::coordinator::CoordinatorBuilder;
pub use self::coordinator::ICoordinated;
pub use self::factory::ILeaseRegistry;
pub use self::factory::LeaseFactory;
pub use self::factory::LeaseFactorySyncArgs;
pub use self::factory::LeaseRegistry;
pub use self::factory::LeaseRegistryArgs;
pub use self::lease::ILease;
pub use self::lease::Lease;
pub use self::lease::LeaseBuilder;
pub use self::lease::LeaseHandle;
pub use self::lease::State;
pub use self::lock::locked;
pub use self::lock::LockAbandoned;
pub use self::time::ICoordinatedTimer;
pub use self::time::Timer;

#[cfg(any(test, feature = "test-fixture"))]
mod fixture;

#[cfg(any(test, feature = "test-fixture"))]
pub use self::fixture::{
    CoordinatedFixture, CoordinatedFixtureNotification, CoordinatedFixtureNotifications,
    FixedLeaseCallback, ICallback as ILeaseFixtureCallback, LeaseFixture, LeaseFixtureNotification,
    LeaseFixtureNotifications,
};
