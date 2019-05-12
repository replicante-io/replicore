use std::sync::Mutex;

use slog::Drain;
use slog::IgnoreResult;
use slog::Logger;

use slog::FnValue;
use slog::Record;

use slog_term::FullFormat;
use slog_term::TermDecorator;

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
        record: &::slog::Record,
        logger_values: &::slog::OwnedKVList,
    ) -> Result<Self::Ok, Self::Err> {
        if record.level().is_at_least(self.1) {
            self.0.log(record, logger_values)?;
        }
        Ok(())
    }
}

arg_enum! {
    /// Enumerate valid log verbosity levels.
    #[derive(Clone, Eq, PartialEq, Hash, Debug)]
    pub enum LogLevel {
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

/// Configure the logger.
pub fn configure(level: LogLevel) -> Logger {
    let decorator = TermDecorator::new().stdout().build();
    let drain = FullFormat::new(decorator).build();
    let drain = Mutex::new(drain).map(IgnoreResult::new);
    let drain = LevelFilter(drain, level.into());
    // rustc can't infer lifetimes correctly when using Record::module.
    // Without this allow, clipply complainants that we do not use Record::module.
    #[allow(clippy::redundant_closure)]
    Logger::root(
        drain,
        o!(
            "module" => FnValue(|rinfo : &Record| rinfo.module()),
            "version" => env!("GIT_BUILD_HASH"),
        ),
    )
}
