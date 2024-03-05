//! Handle converging the known state of the cluster towards the declared version.
use anyhow::Result;
use once_cell::sync::Lazy;

use replicore_context::Context;

mod node_scale_up;
mod step;

use self::step::ConvergeStep;
use crate::init::InitData;

/// Ordered list of cluster convergence steps to perform.
static STEPS: Lazy<Vec<Box<dyn ConvergeStep>>> = Lazy::new(|| {
    vec![
        Box::new(self::node_scale_up::NodeScaleUp),
        // TODO: Cluster init check.
        // TODO: Node joining check (how to choose add or join?).
        // TODO: Node scale down check.
        // TODO: Node replacement check.
    ]
});

/// Process cluster convergence steps in priority order.
pub async fn run(context: &Context, data: &InitData) -> Result<()> {
    for step in STEPS.iter() {
        // TODO: add error to the orchestration report but continue.
        step.converge(context, data).await?;
    }
    Ok(())
}
