//! Wrap futures into a guard that abandons work if a lease is lost.
use std::future::Future;
use std::time::Duration;

use anyhow::Result;

use crate::Lease;
use crate::State;

/// Locked work abandoned as lease is not primary.
#[derive(Debug, thiserror::Error)]
#[error("locked work abandoned as lease '{0}' is not primary")]
pub struct LockAbandoned(String);

impl LockAbandoned {
    /// Initialise a new error with a lease ID.
    pub fn new<S>(id: S) -> Self
    where
        S: Into<String>,
    {
        LockAbandoned(id.into())
    }
}

/// Only execute `work` if the lease is primary, stopping on loss.
///
/// This function takes a newly created lease and a work future:
///
/// - The future is stared only if the lease enters the primary state.
/// - Should the lease ever exit the primary state, work is abandoned.
///
/// If the provided lease does not transition into Primary at the start, the function locks forever.
/// In other works, this function does not support re-using leases.
pub async fn locked<F>(mut lease: Lease, work: F) -> Result<F::Output>
where
    F: Future,
{
    let state = lease.watch().await?;
    if !matches!(state, State::Primary) {
        let error = LockAbandoned::new(lease.id());
        anyhow::bail!(error)
    }

    let result = tokio::select! {
        _ = lease.watch() => {
            let error = LockAbandoned::new(lease.id());
            Err(anyhow::anyhow!(error))
        }
        result = work => Ok(result),
    };

    // Step down the lease to ensure it is release since we are done.
    // As with other cases of "step down to release", the delay is irrelevant.
    lease.step_down(Duration::from_secs(2)).await?;
    result
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::locked;
    use super::LockAbandoned;
    use crate::LeaseFixtureNotification;
    use crate::State;

    #[tokio::test]
    async fn lease_is_primary() {
        let context = replicore_context::Context::fixture();
        let lease = crate::LeaseFixture::fixed(State::Primary);
        let notifs = lease.notifications();
        let lease = crate::Lease::new(context, "test", lease);

        let start = std::time::Instant::now();
        let result = locked(lease, async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            42
        })
        .await
        .unwrap();

        let time = start.elapsed();
        assert_eq!(result, 42);
        assert!(time.as_millis() >= 10, "locked work ended too soon");

        let notifs = notifs.snapshot();
        assert_eq!(
            *notifs.last().unwrap(),
            LeaseFixtureNotification::SteppedDown
        );
    }

    #[tokio::test]
    async fn lease_is_secondary() {
        let context = replicore_context::Context::fixture();
        let lease = crate::LeaseFixture::fixed(State::Secondary);
        let notifs = lease.notifications();
        let lease = crate::Lease::new(context, "test", lease);

        let start = std::time::Instant::now();
        let result = locked(lease, async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            42
        })
        .await;

        let time = start.elapsed();
        match result {
            Err(error) if error.is::<LockAbandoned>() => (),
            Err(error) => panic!("unexpected error from locked: {error:?}"),
            Ok(value) => panic!("unexpected ok from locked: {value:?}"),
        }
        assert!(time.as_millis() < 4, "locked work took too long");

        let notifs = notifs.snapshot();
        assert_eq!(
            *notifs.last().unwrap(),
            LeaseFixtureNotification::Watched(State::Secondary)
        );
    }

    #[tokio::test]
    async fn lease_is_lost_while_working() {
        let context = replicore_context::Context::fixture();
        let lease = crate::LeaseFixture::simple_transition(State::Primary, State::Secondary, 2)
            .set_watch_delay(Duration::from_millis(10));
        let notifs = lease.notifications();
        let lease = crate::Lease::new(context, "test", lease);

        let start = std::time::Instant::now();
        let result = locked(lease, async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            42
        })
        .await;

        let time = start.elapsed();
        match result {
            Err(error) if error.is::<LockAbandoned>() => (),
            Err(error) => panic!("unexpected error from locked: {error:?}"),
            Ok(value) => panic!("unexpected ok from locked: {value:?}"),
        }
        assert!(time.as_millis() >= 20, "locked work ended too soon");
        assert!(time.as_millis() < 30, "locked work took too long");

        let notifs = notifs.snapshot();
        assert_eq!(
            *notifs.last().unwrap(),
            LeaseFixtureNotification::SteppedDown
        );
    }
}
