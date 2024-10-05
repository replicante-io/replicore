//! Interface for a convergence step interface.
use std::collections::HashMap;
use std::time::Duration;

use anyhow::Result;
use time::OffsetDateTime;

use replicore_cluster_models::ConvergeState;
use replicore_context::Context;

use super::ConvergeData;

/// Interface to check the current known state of the cluster and schedule necessary actions.
#[async_trait::async_trait]
pub trait ConvergeStep: Send + Sync {
    /// Check a cluster and schedule convergence actions if needed.
    async fn converge(
        &self,
        context: &Context,
        data: &ConvergeData,
        state: &mut ConvergeState,
    ) -> Result<()>;
}

/// Check the grace period of a convergence step.
///
/// Returns `true` if the step is currently in the grace period.
pub fn grace_check(
    step_id: &str,
    graces: &HashMap<String, OffsetDateTime>,
    grace_time: u64,
) -> bool {
    let grace = match graces.get(step_id) {
        None => return false,
        Some(grace) => grace,
    };
    let grace_time = Duration::from_secs(grace_time * 60);
    let grace_expire = *grace + grace_time;
    grace_expire > time::OffsetDateTime::now_utc()
}

/// Update the [`ConvergeData::graces`] to start the grace period for a step.
pub fn grace_start<S>(step_id: S, graces: &mut HashMap<String, OffsetDateTime>)
where
    S: Into<String>,
{
    graces.insert(step_id.into(), time::OffsetDateTime::now_utc());
}
