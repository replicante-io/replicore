//! Errors encountered during API requests or reported by the remote server.
use anyhow::Context;
use anyhow::Result;
use reqwest::Response;
use serde::de::DeserializeOwned;
use serde_json::Value as Json;

/// The client sent and invalid API request
#[derive(Debug, thiserror::Error)]
#[error("the client sent and invalid API request")]
pub struct ClientError;

/// The server returned an empty API response.
#[derive(Debug, thiserror::Error)]
#[error("the server returned an empty API response")]
pub struct EmptyResponse;

/// Invalid API response received.
#[derive(Debug, thiserror::Error)]
#[error("invalid API response received: {response}")]
pub struct InvalidResponse {
    pub response: String,
}

/// The resource is not available, or access to it is restricted.
#[derive(Debug, thiserror::Error)]
#[error("the resource is not available, or access to it is restricted")]
pub struct ResourceNotFound;

/// Error refers to resource with ID.
#[derive(Debug, thiserror::Error)]
#[error("error refers to {resource} '{id}'")]
pub struct ResourceIdentifier {
    /// Identifier of a resource the error refers to.
    pub id: String,

    /// Type of resource the error refers to.
    pub resource: String,
}

impl ResourceIdentifier {
    /// Resource identifier context for the given resource type and id.
    pub fn reference<S1, S2>(resource: S1, id: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        ResourceIdentifier {
            id: id.into(),
            resource: resource.into(),
        }
    }
}

/// The server failed to process the API request.
#[derive(Debug, thiserror::Error)]
#[error("the server failed to process the API request")]
pub struct ServerError;

/// Decode the body of an HTTP response and correctly handle errors in the process.
pub async fn inspect<T>(response: Response) -> Result<Option<T>>
where
    T: DeserializeOwned,
{
    let code = response.status();
    let text = response.text().await?;

    // Expect 404 errors to not have a response body.
    if matches!(code, reqwest::StatusCode::NOT_FOUND) {
        anyhow::bail!(ResourceNotFound);
    }

    // On error, attempt to decode a JSON object and convert into appropriate errors.
    if code.is_client_error() || code.is_server_error() {
        // For non-validation errors decode the error from the JSON payload.
        let error = serde_json::from_str::<Json>(&text).map_err(|error| {
            let response = text.clone();
            let decode = InvalidResponse { response };
            anyhow::anyhow!(error).context(decode)
        })?;
        let error =
            replisdk::utils::error::from_json(error).context(InvalidResponse { response: text })?;
        let error = match code.is_client_error() {
            true => error.context(ClientError),
            false => error.context(ServerError),
        };
        return Err(error);
    }

    // On success decode the payload, if any, into the requested type.
    if text.is_empty() {
        return Ok(None);
    }
    serde_json::from_str::<T>(&text)
        .map_err(|error| {
            let decode = InvalidResponse { response: text };
            anyhow::anyhow!(error).context(decode)
        })
        .map(Some)
}
