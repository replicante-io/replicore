use slog::error;
use slog::warn;
use slog::Logger;

use replicante_externals_mongodb::admin::ValidationResult;

const GROUP_PERF_ABUSE: &str = "perf/abuse";

/// Collection of outcomes for a set of checks.
#[derive(Clone, Default, Eq, PartialEq, Hash, Debug)]
pub struct Outcomes {
    errors: Vec<Error>,
    errors_count: u64,
    warnings: Vec<Warning>,
    warnings_count: u64,
}

impl Outcomes {
    pub fn new() -> Outcomes {
        Outcomes::default()
    }

    /// Record an error.
    pub fn error(&mut self, error: Error) {
        self.errors_count += 1;
        self.errors.push(error)
    }

    /// Return true if there are error outcomes.
    pub fn has_errors(&self) -> bool {
        self.errors_count > 0
    }

    /// Return true if there are warning outcomes.
    pub fn has_warnings(&self) -> bool {
        self.warnings_count > 0
    }

    /// Logs all the collected warnings and errors.
    pub fn report(&mut self, logger: &Logger) {
        for warn in &self.warnings {
            warn.emit(logger);
        }
        for error in &self.errors {
            error.emit(logger);
        }
        self.errors.clear();
        self.warnings.clear();
    }

    /// Record a warning.
    pub fn warn(&mut self, warning: Warning) {
        self.warnings_count += 1;
        self.warnings.push(warning)
    }
}

/// Enumerate possible check errors.
///
/// Errors are issues that will prevent replicante from working correctly
/// and must be fixed for the system to behave as expected.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Error {
    /// An error was encountered while cheking the system.
    GenericError(String),

    /// The store validator reported an error with the current configuration.
    StoreValidationError(ValidationResult),

    /// A model failed to decode, likely because the format has changed.
    ///
    /// Parameters: model kind, id, message.
    UnableToParseModel(String, String, String),
}

impl Error {
    /// Logs the error.
    pub fn emit(&self, logger: &Logger) {
        let group = self.group();
        match *self {
            Error::GenericError(ref msg) => error!(
                logger, "Check failed with error: {}", msg; "group" => group
            ),
            Error::StoreValidationError(ref result) => error!(
                logger,
                "The store validator reported an error with the current configuration: {}",
                result.message; "group" => group, "collection" => &result.collection
            ),
            Error::UnableToParseModel(ref kind, ref id, ref msg) => error!(
                logger, "Fail to decode a '{}': {}", kind, msg;
                "group" => group, "model" => kind, "id" => id
            ),
        };
    }

    /// Issue group for the error.
    pub fn group(&self) -> &'static str {
        match *self {
            Error::GenericError(_) => "generic/error",
            Error::StoreValidationError(ref result) => result.group,
            Error::UnableToParseModel(_, _, _) => "data/format",
        }
    }
}

/// Enumerate possible check warnings.
///
/// Warnings are issues that will NOT prevent replicante from working correctly
/// but may lead to poor performance, instability, or other similar risk.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Warning {
    /// A configuration option is below the suggested threshold.
    ///
    /// Parameters: message, current, threshold.
    BelowThreshold(String, u64, u64),

    /// The store validator reported an issue or had a suggestion.
    StoreValidationWarning(ValidationResult),
}

impl Warning {
    /// Logs the warning.
    pub fn emit(&self, logger: &Logger) {
        let group = self.group();
        match *self {
            Warning::BelowThreshold(ref message, ref current, ref threshold) => warn!(
                logger, "Value is below recommended threshold: {}", message;
                "current" => current, "threshold" => threshold, "group" => group
            ),
            Warning::StoreValidationWarning(ref result) => warn!(
                logger, "The store validator reported an issue or had a suggestion: {}",
                result.message; "group" => group, "collection" => &result.collection
            ),
        };
    }

    /// Issue group for the warning.
    pub fn group(&self) -> &'static str {
        match *self {
            Warning::BelowThreshold(_, _, _) => GROUP_PERF_ABUSE,
            Warning::StoreValidationWarning(ref result) => result.group,
        }
    }
}
