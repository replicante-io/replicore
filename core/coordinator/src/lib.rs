//! Replicante Control Plane distributed coordinator for exclusive logic execution.
//!
//! This service provides a framework for Replicante Core logic to ensure logic execution
//! happens only once at the same time in the cluster, regardless of how many nodes are involved.

mod lease;

pub use self::lease::ILease;
pub use self::lease::Lease;
pub use self::lease::State;

#[cfg(any(test, feature = "test-fixture"))]
pub use self::lease::fixture::{
    ICallback as ILeaseFixtureCallback, LeaseFixture, LeaseFixtureNotification,
    LeaseFixtureNotifications,
};
