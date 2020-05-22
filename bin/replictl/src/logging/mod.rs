use anyhow::Result;
use slog::Logger;
use structopt::clap::arg_enum;
use structopt::StructOpt;

use crate::utils::resolve_home;

mod logger;

arg_enum! {
    /// Enumerate valid log verbosity levels.
    #[derive(Clone, Debug)]
    enum LogLevel {
        Critical,
        Error,
        Warning,
        Info,
        Debug,
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
#[derive(StructOpt, Debug)]
pub struct LogOpt {
    /// If provided, logs will be emitted to this file.
    #[structopt(long = "log-file", name = "log-file", global = true)]
    file: Option<String>,

    /// Verbosity level for the log file.
    #[structopt(
        long = "log-level", case_insensitive = true, global = true,
        default_value = "info", possible_values = &LogLevel::variants()
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
