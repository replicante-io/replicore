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
    /// The drain to send logs to.
    #[serde(default)]
    drain: LoggingDrain,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            drain: LoggingDrain::default(),
        }
    }
}


/// Converts a [`Drain`] into a [`Logger`] setting global tags. 
///
/// [`Drain`]: slog/trait.Drain.html
/// [`Logger`]: slog/struct.Logger.html
fn drain_into_logger<D>(drain: D) -> Logger
    where D: SendSyncUnwindSafeDrain<Ok = (), Err = Never>,
          D: 'static + SendSyncRefUnwindSafeDrain<Err = Never, Ok = ()>
{
    Logger::root(drain, o!(
        "version" => env!("GIT_BUILD_HASH")
    ))
}


/// Creates a [`Logger`] based on the given configuration.
///
/// [`Logger`]: slog/struct.Logger.html
pub fn configure(config: Config) -> Logger {
    let drain = match config.drain {
        LoggingDrain::Json => Mutex::new(Json::default(stdout())).map(IgnoreResult::new),
    };
    let drain = Async::new(drain).build().ignore_res();
    drain_into_logger(drain)
}

/// Creates a fixed [`Logger`] to be used until configuration is loaded.
///
/// [`Logger`]: slog/struct.Logger.html
pub fn starter() -> Logger {
    let drain = Mutex::new(Json::default(stdout())).map(IgnoreResult::new);
    drain_into_logger(drain)
}
