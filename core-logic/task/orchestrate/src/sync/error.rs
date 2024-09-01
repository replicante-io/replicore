//! Errors Synchronising the state of nodes with the control plane.
use anyhow::Error;
use anyhow::Result;

/// Marker to indicate node sync error that should not prevent processing of other nodes.
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct NodeSpecificError {
    #[from]
    inner: Error,
}

impl NodeSpecificError {
    pub fn wrap<E>(error: E) -> Error
    where
        E: Into<Error>,
    {
        let error = NodeSpecificError {
            inner: error.into(),
        };
        Error::from(error)
    }
}

/// Helper trait to verify if errors are node specific or sync wide.
///
/// The aim of this trait is to keep cluster sync code cleaner and easier to read.
/// Errors are categorised as:
///
/// - Node specific: syncing a specific node failed but the cluster sync process should continue.
///   Examples include: network issues or crashed nodes.
/// - Sync wide: syncing failed in a way that will impact the whole cluster sync process.
///   Examples include: DB issues.
pub trait NodeSpecificCheck<T> {
    /// Classify errors into either node specific or sync wide.
    ///
    /// - Node specific errors are returned as `Ok(Err(error))`.
    /// - Sync wide errors are returned as `Err(error)`.
    /// - Successes are returned as `Ok(Ok(value))`.
    fn with_node_specific(self) -> Result<Result<T>>;
}

impl<T> NodeSpecificCheck<T> for Result<T> {
    fn with_node_specific(self) -> Result<Result<T>> {
        // Ignore non-error values as we only select error types.
        let error = match self {
            Err(error) => error,
            Ok(value) => return Ok(Ok(value)),
        };

        // Return node specific errors to the caller for processing.
        if any_is::<NodeSpecificError>(&error) {
            return Ok(Err(error));
        }

        // Any other errors should interrupt the caller.
        Err(error)
    }
}

/// Is the error or any error in the source chain of parameter type `T`?
fn any_is<T>(error: &Error) -> bool
where
    T: std::error::Error + 'static,
    T: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
{
    error.is::<T>() || error.chain().any(|e| e.is::<T>())
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::NodeSpecificCheck;

    #[derive(thiserror::Error, Debug)]
    #[error("layered errors are wrapped only if they wrap a node error")]
    struct LayeredError(#[source] anyhow::Error);

    #[test]
    fn propagate_unknown_errors() {
        let value: Result<bool> = Err(anyhow::anyhow!("this error should fail fast"));
        let result = value.with_node_specific();
        let error = match result {
            Err(error) => error,
            Ok(_) => panic!("expected the error to propagate"),
        };
        assert_eq!(error.to_string(), "this error should fail fast");
    }

    #[test]
    fn wrap_success() {
        let value = Ok(42);
        let result = value
            .with_node_specific()
            .expect("the ok value to be double wrapped in a result")
            .expect("the ok value to be single wrapped in a result");
        assert_eq!(result, 42);
    }

    #[test]
    fn wrap_chained_node_errors() {
        // Layered node error.
        let value = anyhow::anyhow!("starting error");
        let value = super::NodeSpecificError::wrap(value);
        let value = LayeredError(anyhow::anyhow!(value));
        let value: Result<bool> = Err(anyhow::anyhow!(value));
        let result = value
            .with_node_specific()
            .expect("the error to be double wrapped in a result");
        let error = match result {
            Err(error) => error,
            Ok(_) => panic!("expected the inner error to propagate"),
        };
        assert_eq!(
            error.to_string(),
            "layered errors are wrapped only if they wrap a node error"
        );

        // Layered non node error.
        let value = LayeredError(anyhow::anyhow!("not a node error"));
        let value: Result<bool> = Err(anyhow::anyhow!(value));
        let result = value.with_node_specific();
        let error = match result {
            Err(error) => error,
            Ok(_) => panic!("expected the error to propagate"),
        };
        assert_eq!(
            error.to_string(),
            "layered errors are wrapped only if they wrap a node error"
        );
    }

    #[test]
    fn wrap_contexted_node_errors() {
        // Layered non none error.
        let value: Result<bool> = Err(anyhow::anyhow!("not a sync error"));
        let value = value.map_err(super::NodeSpecificError::wrap);
        let result = value
            .with_node_specific()
            .expect("the error to be double wrapped in a result");
        let error = match result {
            Err(error) => error,
            Ok(_) => panic!("expected the inner error to propagate"),
        };
        assert_eq!(error.to_string(), "not a sync error");
    }

    #[test]
    fn wrap_node_errors() {
        let value = anyhow::anyhow!("starting error");
        let value = super::NodeSpecificError::wrap(value);
        let value: Result<bool> = Err(anyhow::anyhow!(value));
        let result = value
            .with_node_specific()
            .expect("the error to be double wrapped in a result");
        let error = match result {
            Err(error) => error,
            Ok(_) => panic!("expected the inner error to propagate"),
        };
        assert_eq!(error.to_string(), "starting error");
    }
}
