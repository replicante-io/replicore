use std::fs::OpenOptions;
use std::sync::Mutex;

use clap::arg_enum;
use clap::value_t;
use clap::App;
use clap::Arg;
use clap::ArgMatches;
use failure::ResultExt;
use slog::o;
use slog::Discard;
use slog::Drain;
use slog::FnValue;
use slog::IgnoreResult;
use slog::Logger;
use slog::OwnedKVList;
use slog::Record;

use crate::utils::resolve_home;
use crate::ErrorKind;
use crate::Result;

/// Alternative implementation of slog's [`LevelFilter`] with `Ok == ()`.
///
/// The default [`LevelFilter`] implementation wraps `D::Ok` into an [`Option`].
/// This makes it impossible to wrap a filtering drain into a [`Logger`].
///
/// [`LevelFilter`]: slog/struct.LevelFilter.html
/// [`Logger`]: slog/struct.Logger.html
/// [`Option`]: core/option/enum.Option.html
#[derive(Debug, Clone)]
struct LevelFilter<D: Drain>(pub D, pub ::slog::Level);
impl<D: Drain> Drain for LevelFilter<D> {
    type Ok = ();
    type Err = D::Err;
    fn log(
        &self,
        record: &Record,
        logger_values: &OwnedKVList,
    ) -> std::result::Result<Self::Ok, Self::Err> {
        if record.level().is_at_least(self.1) {
            self.0.log(record, logger_values)?;
        }
        Ok(())
    }
}

arg_enum! {
    /// Enumerate valid log verbosity levels.
    #[derive(Clone, Eq, PartialEq, Hash, Debug)]
    enum LogLevel {
        Critical,
        Error,
        Warning,
        Info,
        Debug,
    }
}

impl Default for LogLevel {
    #[cfg(debug_assertions)]
    fn default() -> LogLevel {
        LogLevel::Debug
    }

    #[cfg(not(debug_assertions))]
    fn default() -> LogLevel {
        LogLevel::Info
    }
}

impl From<LogLevel> for ::slog::Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Critical => ::slog::Level::Critical,
            LogLevel::Error => ::slog::Level::Error,
            LogLevel::Warning => ::slog::Level::Warning,
            LogLevel::Info => ::slog::Level::Info,
            LogLevel::Debug => ::slog::Level::Debug,
        }
    }
}

/// Initialise a logger based on the given CLI arguments.
pub fn configure<'a>(cli: &ArgMatches<'a>) -> Result<Logger> {
    // Load CLI parameters.
    let file = match cli.value_of("log-file") {
        Some(file) => file,
        None => return Ok(Logger::root(Discard {}, o!())),
    };
    let file = resolve_home(file)?;
    let level = value_t!(cli, "log-level", LogLevel).unwrap_or_default();

    // Setup JSON logging to a file.
    let writer = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(file.clone())
        .with_context(|_| ErrorKind::FsOpen(file))?;
    let drain = slog_json::Json::new(writer)
        .set_newlines(true)
        .set_flush(true)
        .set_pretty(false)
        .add_default_keys()
        .build();
    let drain = Mutex::new(drain).map(IgnoreResult::new);
    let drain = LevelFilter(drain, level.into());
    Ok(Logger::root(
        drain,
        o!(
            "module" => FnValue(|rinfo : &Record| rinfo.module()),
        ),
    ))
}

/// Configure the given `clap::App` with logging related options.
pub fn configure_cli<'a, 'b>(cli: App<'a, 'b>) -> App<'a, 'b> {
    cli.arg(
        Arg::with_name("log-file")
            .long("log-file")
            .value_name("FILE")
            .takes_value(true)
            .global(true)
            .help("If provided, logs will be emitted to this file"),
    )
    .arg(
        Arg::with_name("log-level")
            .long("log-level")
            .value_name("LEVEL")
            .takes_value(true)
            .possible_values(&LogLevel::variants())
            .case_insensitive(true)
            .global(true)
            .help("Verbosity level for the log file"),
    )
}
