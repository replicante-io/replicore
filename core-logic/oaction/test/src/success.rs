//! Complete action execution as soon as invoked.
use anyhow::Result;

use replisdk::core::models::oaction::OAction;
use replisdk::core::models::oaction::OActionState;

use replicore_context::Context;
use replicore_oaction::OActionChanges;
use replicore_oaction::OActionHandler;
use replicore_oaction::OActionMetadata;

/// Complete action execution as soon as invoked.
#[derive(Debug)]
pub struct Success;

impl Success {
    /// Registration metadata for the `core.replicante.io/test.success` action.
    pub fn metadata() -> OActionMetadata {
        let mut metadata =
            OActionMetadata::build(format!("{}.success", crate::KIND_PREFIX), Success);
        metadata.timeout(crate::DEFAULT_TIMEOUT);
        metadata.finish()
    }
}

#[async_trait::async_trait]
impl OActionHandler for Success {
    async fn invoke(&self, _: &Context, _: &OAction) -> Result<OActionChanges> {
        Ok(OActionChanges::to(OActionState::Done))
    }
}
