use anyhow::Context;
use anyhow::Result;
use serde::de::Deserializer;
use serde::Deserialize;
use serde::Serialize;

use replicante_models_core::actions::orchestrator::OrchestratorAction;

/// Arguments passed to an HTTP request.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Args {
    /// The remote system to invoke the action against.
    pub remote: RemoteArgs,
}

/// Configuration for the remote to invoke actions against.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RemoteArgs {
    /// Optional PEM encoded certificate authority to add to the client.
    pub ca: Option<String>,

    /// Optional timeout to wait for a response, in seconds.
    ///
    /// Explicitly set to None/null to disable the timeout.
    #[serde(default, deserialize_with = "deserialize_explicit_optional")]
    pub timeout: Option<Option<u64>>,

    /// URL of the remote system to invoke the action against.
    pub url: String,
}

impl Args {
    /// Decode action arguments from the `OrchestratorAction` record.
    pub fn decode(record: &OrchestratorAction) -> Result<Args> {
        serde_json::from_value(record.args.clone())
            .context(crate::errors::InvalidRecord::InvalidArgs)
            .map_err(anyhow::Error::from)
    }
}

/// Deserialize an optional field distinguishing between missing and explicit null.
fn deserialize_explicit_optional<'de, T, D>(
    deserializer: D,
) -> std::result::Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Some(Option::deserialize(deserializer)?))
}
