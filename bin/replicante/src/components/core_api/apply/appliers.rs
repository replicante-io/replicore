use serde_json::Value;

use replicante_models_core::api::apply::ApplyObject;

use super::agent_action;
use crate::Result;

const APIV_REPLI_V0: &str = "replicante.io/v0";
const KIND_AGENT_ACTION: &str = "AgentAction";

/// Type of closure that handles a specific `kind` for a specific `apiVersion`.
pub type Applier = Box<dyn Fn(ApplierArgs) -> Result<Value>>;

/// Data object that collects arguments passed to `Applier`s.
pub struct ApplierArgs {
    pub object: ApplyObject,
}

/// Find an `Applier` for the given object, if one is implemented.
pub fn find(object: &ApplyObject) -> Option<Applier> {
    let api_version = object.api_version.as_str();
    let kind = object.kind.as_str();
    match (api_version, kind) {
        (APIV_REPLI_V0, KIND_AGENT_ACTION) => Some(Box::new(agent_action::replicante_io_v0)),
        _ => None,
    }
}
