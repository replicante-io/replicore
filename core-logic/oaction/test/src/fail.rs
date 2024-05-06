//! Fail action execution as soon as invoked.
use anyhow::Result;

use replisdk::core::models::oaction::OAction;
use replisdk::core::models::oaction::OActionState;

use replicore_context::Context;
use replicore_oaction::OActionChanges;
use replicore_oaction::OActionHandler;
use replicore_oaction::OActionMetadata;

/// Fail action execution as soon as invoked.
#[derive(Debug)]
pub struct Fail;

impl Fail {
    /// Registration metadata for the `core.replicante.io/test.fail` action.
    pub fn metadata() -> OActionMetadata {
        let mut metadata = OActionMetadata::build(format!("{}.fail", crate::KIND_PREFIX), Fail);
        metadata.timeout(crate::DEFAULT_TIMEOUT);
        metadata.finish()
    }
}

#[async_trait::async_trait]
impl OActionHandler for Fail {
    async fn invoke(&self, _: &Context, _: &OAction) -> Result<OActionChanges> {
        let error = anyhow::anyhow!("debug action failed as expected");
        let error = replisdk::utils::error::into_json(error);
        Ok(OActionChanges::to(OActionState::Failed).error(error))
    }
}
