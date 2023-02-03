use thiserror::Error;

/// Errors relating to or interacting with a Platform instance.
#[derive(Debug, Error)]
pub enum Platform {
    #[error("the referenced platform is not active")]
    NotActive,

    #[error("unable to find the referenced platform")]
    NotFound,
}
