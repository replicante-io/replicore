use slog::error;
use slog::warn;
use slog::Logger;

use replicante_externals_mongodb::admin::ValidationResult;

const GROUP_PERF_ABUSE: &str = "perf/abuse";

/// Collection of outcomes for a set of checks.
#[derive(Clone, Default, Eq, PartialEq, Hash, Debug)]
pub struct Outcomes {
    errors: Vec<Error>,
    warnings: Vec<Warning>,

    // Track most sever outcome separatly.
    // This is because Outcomes::report flushes the arrays for incremental reports.
    seen_errors: bool,
    seen_warnings: bool,
}

impl Outcomes {
    pub fn new() -> Outcomes {
        Outcomes::default()
    }

    /// Extend these outcomes by consuming another `Outcomes` set.
    pub fn extend(&mut self, other: Outcomes) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self.seen_errors |= other.seen_errors;
        self.seen_warnings |= other.seen_warnings;
    }

    /// Record an error.
    pub fn error(&mut self, error: Error) {
        self.errors.push(error);
        self.seen_errors = true;
    }

    /// Return true if there are error outcomes.
    pub fn has_errors(&self) -> bool {
        self.seen_errors
    }

    /// Return true if there are warning outcomes.
    pub fn has_warnings(&self) -> bool {
        self.seen_warnings
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
        self.warnings.push(warning);
        self.seen_warnings = true;
    }
}

/// Enumerate possible check errors.
///
/// Errors are issues that will prevent replicante from working correctly
/// and must be fixed for the system to behave as expected.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Error {
    /// An error was encountered while cheking the system.
    Generic(String),

    /// The store validator reported an error with the current configuration.
    StoreValidation(ValidationResult),

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
            Error::Generic(ref msg) => error!(
                logger, "Check failed with error: {}", msg; "group" => group
            ),
            Error::StoreValidation(ref result) => error!(
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
            Error::Generic(_) => "generic/error",
            Error::StoreValidation(ref result) => result.group,
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
