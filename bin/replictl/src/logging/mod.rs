use anyhow::Result;
use clap::Args;
use clap::ValueEnum;
use slog::Logger;

use crate::utils::resolve_home;

mod logger;

/// Enumerate valid log verbosity levels.
#[derive(Clone, Debug, ValueEnum)]
enum LogLevel {
    Critical,
    Error,
    Warning,
    Info,
    Debug,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Critical => write!(f, "critical"),
            Self::Error => write!(f, "error"),
            Self::Warning => write!(f, "warning"),
            Self::Info => write!(f, "info"),
            Self::Debug => write!(f, "debug"),
        }
    }
}

impl From<LogLevel> for slog::Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Critical => slog::Level::Critical,
            LogLevel::Error => slog::Level::Error,
            LogLevel::Warning => slog::Level::Warning,
            LogLevel::Info => slog::Level::Info,
            LogLevel::Debug => slog::Level::Debug,
        }
    }
}

/// Logging-related options.
#[derive(Args, Debug)]
pub struct LogOpt {
    /// If provided, logs will be emitted to this file.
    #[arg(long = "log-file", name = "log-file", global = true)]
    file: Option<String>,

    /// Verbosity level for the log file.
    #[arg(
        long = "log-level", global = true,
        default_value_t = LogLevel::Info,
    )]
    level: LogLevel,
}

/// Initialise a logger based on the given CLI arguments.
pub fn configure(opt: &LogOpt) -> Result<Logger> {
    // Load CLI parameters.
    let file = match &opt.file {
        Some(file) => resolve_home(file)?,
        None => return Ok(self::logger::null()),
    };
    let level = opt.level.clone().into();
    self::logger::file(file, level)
}
