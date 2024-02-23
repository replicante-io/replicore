//! Implementation of clusters and node discovery from Platform integrations.
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde::Serialize;

use replicore_tasks::conf::Queue;
use replicore_tasks::submit::TaskSubmission;

mod callback;
mod clients;
mod discover;
mod errors;

pub mod events;

#[cfg(test)]
mod tests;

pub use self::callback::Callback;

/// Background task queue for platform discovery requests.
pub static DISCOVERY_QUEUE: Lazy<Queue> = Lazy::new(|| Queue {
    queue: String::from("platform_discovery"),
    retry_count: 1,
    retry_timeout: std::time::Duration::from_secs(5),
});

/// Request clusters and nodes discovery from platform integrations.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DiscoverPlatform {
    /// ID of the namespace the platform is defined in.
    pub ns_id: String,

    /// Name of the platform to discover cluster snd nodes from.
    pub name: String,
}

impl DiscoverPlatform {
    pub fn new<S1, S2>(ns_id: S1, name: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            ns_id: ns_id.into(),
            name: name.into(),
        }
    }
}

impl TryInto<TaskSubmission> for DiscoverPlatform {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<TaskSubmission, Self::Error> {
        TaskSubmission::new(&DISCOVERY_QUEUE, &self)
    }
}
