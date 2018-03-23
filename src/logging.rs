use std::io::stdout;
use std::sync::Mutex;

use slog::Drain;
use slog::IgnoreResult;
use slog::Logger;

use slog::Never;
use slog::SendSyncRefUnwindSafeDrain;
use slog::SendSyncUnwindSafeDrain;

use slog_async::Async;
use slog_json::Json;


/// List of supported logging drains.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub enum LoggingDrain {
    /// Log JSON objects to standard output.
    Json,
}

impl Default for LoggingDrain {
    fn default() -> LoggingDrain {
        LoggingDrain::Json
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
}

impl Default for Config {
    fn default() -> Config {
        Config {
            async: true,
            drain: LoggingDrain::default(),
        }
    }
}

impl Config {
    /// Default value for `async` used by serde.
    fn default_async() -> bool { true }
}


/// Converts a [`Drain`] into a [`Logger`] setting global tags. 
///
/// [`Drain`]: slog/trait.Drain.html
/// [`Logger`]: slog/struct.Logger.html
fn into_logger<D>(drain: D) -> Logger
    where D: SendSyncUnwindSafeDrain<Ok = (), Err = Never>,
          D: 'static + SendSyncRefUnwindSafeDrain<Err = Never, Ok = ()>
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
          D: 'static + SendSyncRefUnwindSafeDrain<Err = Never, Ok = ()>
{
    match config.async {
        true => into_logger(Async::new(drain).build().ignore_res()),
        false => into_logger(drain),
    }
}


/// Creates a [`Logger`] based on the given configuration.
///
/// [`Logger`]: slog/struct.Logger.html
pub fn configure(config: Config) -> Logger {
    match config.drain {
        LoggingDrain::Json => {
            let drain = Mutex::new(Json::default(stdout())).map(IgnoreResult::new);
            config_async(config, drain)
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
