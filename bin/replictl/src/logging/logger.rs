use std::fs::OpenOptions;
use std::sync::Mutex;

use anyhow::Context;
use anyhow::Result;
use slog::o;
use slog::Discard;
use slog::Drain;
use slog::FnValue;
use slog::IgnoreResult;
use slog::Level;
use slog::Logger;
use slog::OwnedKVList;
use slog::Record;

/// A logger to discard all messages.
pub fn null() -> Logger {
    Logger::root(Discard, o!())
}

/// A loggger to write JSON encoded events to a file.
pub fn file(path: String, level: Level) -> Result<Logger> {
    let writer = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("Unable to open log file at {}", path))?;
    let drain = slog_json::Json::new(writer)
        .set_newlines(true)
        .set_flush(true)
        .set_pretty(false)
        .add_default_keys()
        .build();
    let drain = Mutex::new(drain).map(IgnoreResult::new);
    let drain = LevelFilter(drain, level);
    Ok(Logger::root(
        drain,
        o!(
            "module" => FnValue(|rinfo : &Record| rinfo.module()),
        ),
    ))
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
struct LevelFilter<D: Drain>(pub D, pub Level);
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
