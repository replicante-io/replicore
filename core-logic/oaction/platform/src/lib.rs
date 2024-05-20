//! RepliCore orchestration actions to perform platform operations.
use replicore_oaction::OActionMetadata;

mod provision;

/// Shared prefix for all test action kinds.
const KIND_PREFIX: &str = "core.replicante.io/platform";

pub use self::provision::ProvisionNodes;
pub use self::provision::ProvisionNodesArgs;

/// Collection of orchestrator actions metadata for the `core.replicante.io/platform.*` group.
pub fn all() -> impl IntoIterator<Item = OActionMetadata> {
    [ProvisionNodes::metadata()]
}
