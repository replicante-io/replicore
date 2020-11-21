use serde::Deserialize;
use serde::Serialize;

/// Description of a validation error.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct DiscoverySettingsListResponse {
    pub names: Vec<String>,
}
