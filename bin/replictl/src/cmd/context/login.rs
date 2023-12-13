//! Placeholder for future Replicante Control Plane authentication logic.
use anyhow::Result;

use crate::Globals;

/// Placeholder for future Replicante Control Plane authentication logic.
pub async fn run(_: &Globals) -> Result<i32> {
    println!("Replicante Control Plane authentication is not available (or needed) yet");
    Ok(0)
}
