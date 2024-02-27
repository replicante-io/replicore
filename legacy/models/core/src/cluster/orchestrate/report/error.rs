use thiserror::Error;

/// Possible errors encountered while building an OrchestrateReport;
#[derive(Error, Debug)]
pub enum OrchestrateReportError {
    #[error("orchestrate task duration is not valid")]
    InvalidTaskDuration,

    #[error("cannot build an orchestrate report without knowing which cluster it is for")]
    MissingClusterIdentifier,

    #[error("cannot build an orchestrate report without knowing the outcome")]
    MissingOutcome,

    #[error("cannot build an orchestrate report without knowing the task start time")]
    MissingStartTime,
}
