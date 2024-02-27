use thiserror::Error;

/// Errors configuring the HTTP(S) client.
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("unable to create HTTP(S) client")]
    Create,

    #[error("unable to decoded the provided CA PEM certificate")]
    InvalidCA,
}

/// Errors relating to invalid orchestrator action records.
#[derive(Debug, Error)]
pub enum InvalidRecord {
    #[error("unable to decode action arguments")]
    InvalidArgs,
}

/// Errors interacting with the remote system.
#[derive(Debug, Error)]
pub enum RemoteError {
    #[error("unable to send HTTP request to the remote")]
    RequestFailed,

    #[error("unable to read response body as text")]
    ResponseRead,
}
