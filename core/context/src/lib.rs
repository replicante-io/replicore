//! The [`Context`] is a general purpose immutable container to carry scoped values around.
//!
//! Code executed as part of RepliCore processes can access operation scoped values.
//!
//! Contexts are organised into a tree structure:
//!
//! - A root context represents the general process wide scope.
//! - Derived contexts represents a narrower scope within their parent with additional
//!   or updated information attached to them.
//!
//! For example: [`Context`]s provide access to the current [`Logger`].
//! For the root context this is the process-wide logger with no additional attributes.
//! But for individual operations a derived context can be provided with a [`Logger`] decorated
//! with the operation trace ID or other request attributes.
use slog::Logger;
use slog::OwnedKV;
use slog::SendSyncRefUnwindSafeKV;

/// The [`Context`] is a general purpose container to carry scoped values around.
///
/// Refer to the [crate level docs](crate) for details.
#[derive(Clone, Debug)]
pub struct Context {
    /// Logger with contextual attributes attached to it.
    pub logger: Logger,
}

impl Context {
    /// Derive a new [`Context`] by making changes to the current one.
    pub fn derive(&self) -> ContextBuilder {
        ContextBuilder {
            logger: self.logger.clone(),
        }
    }

    /// Derive a new [`Context`] by making changes to the current one using the provided callback.
    pub fn derive_with<F>(&self, callback: F) -> Context
    where
        F: FnOnce(ContextBuilder) -> ContextBuilder,
    {
        let builder = callback(self.derive());
        builder.build()
    }

    /// Initialise a new root context with no values attached.
    pub fn root(logger: Logger) -> ContextBuilder {
        ContextBuilder { logger }
    }
}

/// A builder for root and derived contexts.
pub struct ContextBuilder {
    logger: Logger,
}

impl ContextBuilder {
    /// Finalise the build process and return a new [`Context`].
    pub fn build(self) -> Context {
        Context {
            logger: self.logger,
        }
    }

    /// Update the [`Context`] logger to attach new log key/pair values.
    pub fn log_values<T>(mut self, entries: OwnedKV<T>) -> Self
    where
        T: SendSyncRefUnwindSafeKV + 'static,
    {
        self.logger = self.logger.new(entries);
        self
    }
}

#[cfg(any(test, feature = "test-fixture"))]
impl Context {
    /// Create an empty context useful for test.
    pub fn fixture() -> Context {
        let logger = Logger::root(slog::Discard, slog::o!());
        Context { logger }
    }
}

#[cfg(test)]
mod tests {
    use super::Context;

    #[test]
    fn derive_log_attributes() {
        let root = Context::fixture();
        let parent = root
            .derive()
            .log_values(slog::o!("root" => "value", "test" => "root"))
            .build();
        let context = parent
            .derive()
            .log_values(slog::o!("test" => "override"))
            .build();
        assert_eq!(format!("{:?}", context.logger.list()), "(test, test, root)");
    }

    #[test]
    fn derive_noop() {
        let parent = Context::fixture();
        let context = parent.derive().build();
        assert_eq!(
            format!("{:?}", parent.logger.list()),
            format!("{:?}", context.logger.list()),
        );
    }
}
