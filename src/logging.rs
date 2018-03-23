use std::io::stdout;
use std::sync::Mutex;

use slog::Drain;
use slog::IgnoreResult;
use slog::Logger;

use slog::Never;
use slog::SendSyncRefUnwindSafeDrain;
use slog::SendSyncUnwindSafeDrain;

use slog_async::Async;
#[cfg(feature = "journald")]
use slog_journald::JournaldDrain;
use slog_json::Json;


/// List of supported logging drains.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub enum LoggingDrain {
    /// Log objects to systemd journal (journald).
    #[cfg(feature = "journald")]
    #[sede(rename = "journald")]
    Journald,

    /// Log JSON objects to standard output.
    #[serde(rename = "json")]
    Json,
}

impl Default for LoggingDrain {
    fn default() -> LoggingDrain {
        LoggingDrain::Json
    }
}


/// Possible logging levels.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub enum LoggingLevel {
    /// Critical
    #[serde(rename = "critical")]
    Critical,

    /// Error
    #[serde(rename = "error")]
    Error,

    /// Warning
    #[serde(rename = "warning")]
    Warning,

    /// Info
    #[serde(rename = "info")]
    Info,

    /// Debug
    #[serde(rename = "debug")]
    Debug,
}

impl Default for LoggingLevel {
    fn default() -> LoggingLevel {
        LoggingLevel::Info
    }
}

impl From<LoggingLevel> for ::slog::Level {
    fn from(level: LoggingLevel) -> Self {
        match level {
            LoggingLevel::Critical => ::slog::Level::Critical,
            LoggingLevel::Error => ::slog::Level::Error,
            LoggingLevel::Warning => ::slog::Level::Warning,
            LoggingLevel::Info => ::slog::Level::Info,
            LoggingLevel::Debug => ::slog::Level::Debug,
        }
    }
}


/// Logging configuration options.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Config {
    /// Flush logs asynchronously.
    #[serde(default = "Config::default_async")]
    async: bool,

    /// The drain to send logs to.
    #[serde(default)]
    drain: LoggingDrain,

    /// The minimum logging level.
    #[serde(default)]
    level: LoggingLevel,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            async: true,
            drain: LoggingDrain::default(),
            level: LoggingLevel::default(),
        }
    }
}

impl Config {
    /// Default value for `async` used by serde.
    fn default_async() -> bool { true }
}


/// Alternative implementation of slog's [`LevelFilter`] with `Ok == ()`.
///
/// The default [`LevelFilter`] implementation wraps `D::Ok` into an [`Option`].
/// This makes it impossible to wrap a filtering drain into a [`Logger`].
///
/// [`LevelFilter`]: slog/struct.LevelFilter.html
/// [`Logger`]: slog/struct.Logger.html
/// [`Option`]: core/option/enum.Option.html
#[derive(Debug, Clone)]
pub struct LevelFilter<D: Drain>(pub D, pub ::slog::Level);
impl<D: Drain> Drain for LevelFilter<D> {
    type Ok = ();
    type Err = D::Err;
    fn log(
        &self,
        record: &::slog::Record,
        logger_values: &::slog::OwnedKVList,
    ) -> Result<Self::Ok, Self::Err> {
        if record.level().is_at_least(self.1) {
            self.0.log(record, logger_values)?;
        }
        Ok(())
    }
}


/// Converts a [`Drain`] into a [`Logger`] setting global tags. 
///
/// [`Drain`]: slog/trait.Drain.html
/// [`Logger`]: slog/struct.Logger.html
fn into_logger<D>(drain: D) -> Logger
    where D: SendSyncUnwindSafeDrain<Ok = (), Err = Never>,
          D: 'static + SendSyncRefUnwindSafeDrain<Ok = (), Err = Never>
{
    Logger::root(drain, o!(
        "version" => env!("GIT_BUILD_HASH")
    ))
}

/// Optionally wrap the drain into an [`Async`] drain.
///
/// [`Async`]: slog_async/struct.Async.html
fn config_async<D>(config: Config, drain: D) -> Logger
    where D: SendSyncUnwindSafeDrain<Ok = (), Err = Never>,
          D: 'static + SendSyncRefUnwindSafeDrain<Ok = (), Err = Never>
{
    match config.async {
        true => into_logger(Async::new(drain).build().ignore_res()),
        false => into_logger(drain),
    }
}

/// Configures the desired logging level.
fn config_level<D>(config: Config, drain: D) -> Logger
    where D: SendSyncUnwindSafeDrain<Ok = (), Err = Never>,
          D: 'static + SendSyncRefUnwindSafeDrain<Ok = (), Err = Never>
{
    let drain = LevelFilter(drain, config.level.clone().into());
    config_async(config, drain)
}


/// Creates a [`Logger`] based on the given configuration.
///
/// This is the first function in a list of generic functions.
/// The intermediate configuration stages, while all compatible with the [`Drain`] trait,
/// have different concrete types.
/// Using generic functions allows code reuse without repeatedly boxing intermediate steps.
///
/// When adding calls to the chain the following principle should be considered:
///
///   * Filters should be applied *before* the `config_async` call.
///   * Processing should be applied *after* the `config_async` call.
///
/// [`Drain`]: slog/trait.Drain.html
/// [`Logger`]: slog/struct.Logger.html
pub fn configure(config: Config) -> Logger {
    match config.drain {
        #[cfg(feature = "journald")]
        LoggingDrain::Journald => config_level(config, JournaldDrain.ignore_res()),
        LoggingDrain::Json => {
            let drain = Mutex::new(Json::default(stdout())).map(IgnoreResult::new);
            config_level(config, drain)
        },
    }
}

/// Creates a fixed [`Logger`] to be used until configuration is loaded.
///
/// [`Logger`]: slog/struct.Logger.html
pub fn starter() -> Logger {
    let drain = Mutex::new(Json::default(stdout())).map(IgnoreResult::new);
    into_logger(drain)
}
