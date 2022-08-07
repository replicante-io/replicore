use anyhow::Result;

mod init;
mod operation;
mod sched_choice;
mod sync;

pub use self::init::InitError;
pub use self::operation::OperationError;
pub use self::sched_choice::SchedChoiceError;
pub use self::sync::SyncError;

/// Extension for `anyhow::Result` to enable cleaner checks for orchestration ending errors.
pub(crate) trait OrchestratorEnder<T> {
    /// Wrap non-ending errors into another result to make checking easier.
    fn orchestration_failed(self) -> Result<Result<T>>;
}

impl<T> OrchestratorEnder<T> for Result<T> {
    fn orchestration_failed(self) -> Result<Result<T>> {
        // Pass successes on but grab the errors to check.
        let error = match self {
            Err(error) => error,
            Ok(value) => return Ok(Ok(value)),
        };

        // Check if the error (or any of its causes) should NOT fail the whole orchestration.
        if error.is::<SyncError>() {
            return Ok(Err(error));
        }
        for source in error.chain() {
            if source.is::<SyncError>() {
                return Ok(Err(error));
            }
        }

        // All other errors should be propagated immediately.
        Err(error)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use anyhow::Result;

    use super::OrchestratorEnder;

    #[derive(thiserror::Error, Debug)]
    #[error("layered errors are wrapped only is they wrap a known error")]
    struct LayeredError(#[source] anyhow::Error);

    #[test]
    fn orchestrator_ender_propagates_unsafe_errors() {
        let value: Result<bool> = Err(anyhow::anyhow!("this error should fail fast"));
        let result = value.orchestration_failed();
        let error = match result {
            Err(error) => error,
            Ok(_) => panic!("expected the error to propagate"),
        };
        assert_eq!(error.to_string(), "this error should fail fast");
    }

    #[test]
    fn orchestrator_ender_wraps_success() {
        let value = Ok(42);
        let result = value
            .orchestration_failed()
            .expect("the ok value to be double wrapped in a result")
            .expect("the ok value to be single wrapped in a result");
        assert_eq!(result, 42);
    }

    #[test]
    fn orchestrator_ender_wraps_chained_sync_errors() {
        // Layered sync error.
        let value = super::SyncError::client_connect("ns", "cluster", "node");
        let value = LayeredError(anyhow::anyhow!(value));
        let value: Result<bool> = Err(anyhow::anyhow!(value));
        let result = value
            .orchestration_failed()
            .expect("the error to be double wrapped in a result");
        let error = match result {
            Err(error) => error,
            Ok(_) => panic!("expected the inner error to propagate"),
        };
        assert_eq!(
            error.to_string(),
            "layered errors are wrapped only is they wrap a known error"
        );

        // Layered non sync error.
        let value = LayeredError(anyhow::anyhow!("not a sync error"));
        let value: Result<bool> = Err(anyhow::anyhow!(value));
        let result = value.orchestration_failed();
        let error = match result {
            Err(error) => error,
            Ok(_) => panic!("expected the error to propagate"),
        };
        assert_eq!(
            error.to_string(),
            "layered errors are wrapped only is they wrap a known error"
        );
    }

    #[test]
    fn orchestrator_ender_wraps_contexted_sync_errors() {
        // Layered non sync error.
        let value: Result<bool> = Err(anyhow::anyhow!("not a sync error"));
        let value =
            value.with_context(|| super::SyncError::client_connect("ns", "cluster", "node"));
        let result = value
            .orchestration_failed()
            .expect("the error to be double wrapped in a result");
        let error = match result {
            Err(error) => error,
            Ok(_) => panic!("expected the inner error to propagate"),
        };
        assert_eq!(
            error.to_string(),
            "failed to connect to node node in cluster ns.cluster"
        );
    }

    #[test]
    fn orchestrator_ender_wraps_sync_errors() {
        let value = super::SyncError::client_connect("ns", "cluster", "node");
        let value: Result<bool> = Err(anyhow::anyhow!(value));
        let result = value
            .orchestration_failed()
            .expect("the error to be double wrapped in a result");
        let error = match result {
            Err(error) => error,
            Ok(_) => panic!("expected the inner error to propagate"),
        };
        assert_eq!(
            error.to_string(),
            "failed to connect to node node in cluster ns.cluster"
        );
    }
}
