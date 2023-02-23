#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("podman command returned exit code {0}")]
    // (exit_code,)
    CommandFailed(i32),

    #[error("failed to list dependencies")]
    DependenciesList,

    #[error("podman command failed to execute")]
    ExecFailed,

    #[error("file system path not valid unicode")]
    FsNotUnicode,

    #[error("invalid hostname in the HOSTNAME environment variable")]
    InvalidHostnameVar,

    #[error("io error for {0}")]
    IoError(String),

    #[error("unable to find pod {0}")]
    // (pod_id,)
    PodNotFound(String),

    #[error("unable to decode pod specification JSON for {0}")]
    // (pod_id,)
    PodNotValid(String),
}

impl Error {
    /// IO error on path.
    pub fn io_error<P: Into<String>>(path: P) -> Self {
        Self::IoError(path.into())
    }

    /// Could not find requested pod.
    pub fn pod_not_found<P: Into<String>>(pod: P) -> Self {
        Self::PodNotFound(pod.into())
    }

    /// Could not decode pod specification JSON.
    pub fn pod_not_valid<P: Into<String>>(pod: P) -> Self {
        Self::PodNotValid(pod.into())
    }
}
