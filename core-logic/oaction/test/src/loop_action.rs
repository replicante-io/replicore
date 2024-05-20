//! Increment a counter each invocation loop until a total is reached.
use anyhow::Result;

use replisdk::core::models::oaction::OActionState;

use replicore_context::Context;
use replicore_oaction::OActionChanges;
use replicore_oaction::OActionHandler;
use replicore_oaction::OActionInvokeArgs;
use replicore_oaction::OActionMetadata;

/// Increment a counter each invocation loop until a total is reached.
///
/// The total to reach can be specified with `target` key in the [`OAction::args`].
/// A default of 10 is used if the argument is missing or invalid.
///
/// When the target total is reached the action is completed.
#[derive(Debug)]
pub struct Loop;

impl Loop {
    /// Registration metadata for the `core.replicante.io/test.loop` action.
    pub fn metadata() -> OActionMetadata {
        let mut metadata = OActionMetadata::build(format!("{}.loop", crate::KIND_PREFIX), Loop);
        metadata.timeout(crate::DEFAULT_TIMEOUT);
        metadata.finish()
    }
}

#[async_trait::async_trait]
impl OActionHandler for Loop {
    async fn invoke(&self, _: &Context, args: &OActionInvokeArgs) -> Result<OActionChanges> {
        // Get target from the payload, or fallback to args, or fallback to default.
        let target = args
            .action
            .state_payload
            .as_ref()
            .and_then(|payload| payload.as_object())
            .or_else(|| args.action.args.as_object())
            .and_then(|object| object.get("target"))
            .and_then(|target| target.as_u64())
            .unwrap_or(10);
        // Get current count for payload, or fallback to start.
        let count = args
            .action
            .state_payload
            .as_ref()
            .and_then(|payload| payload.as_object())
            .and_then(|payload| payload.get("count"))
            .and_then(|count| count.as_u64())
            .unwrap_or(0);
        let count = count + 1;
        let phase = if count >= target {
            OActionState::Done
        } else {
            OActionState::Running
        };
        let changes = OActionChanges::to(phase).payload(serde_json::json!({
            "count": count,
            "target": target,
        }));
        Ok(changes)
    }
}
