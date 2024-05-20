//! RepliCore orchestration actions to aid testing and system exploration.
use replicore_oaction::OActionMetadata;

mod fail;
mod loop_action;
mod success;

/// Test actions should complete quickly.
const DEFAULT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10 * 60);

/// Shared prefix for all test action kinds.
const KIND_PREFIX: &str = "core.replicante.io/test";

pub use self::fail::Fail;
pub use self::loop_action::Loop;
pub use self::success::Success;

/// Collection of orchestrator actions metadata for the `core.replicante.io/test.*` group.
pub fn all() -> impl IntoIterator<Item = OActionMetadata> {
    [Fail::metadata(), Loop::metadata(), Success::metadata()]
}
