use clap::value_t;
use clap::ArgMatches;
use failure::ResultExt;
use slog::info;
use slog::Logger;

use super::ErrorKind;
use super::Result;

/// A container sturcture to inject dependencies.
pub struct Interfaces {
    logger: Logger,

    // Internal attributes.
    progress: bool,
    progress_chunk: u32,
}

impl Interfaces {
    /// Create a new `Interfaces` container.
    pub fn new<'a>(args: &ArgMatches<'a>, logger: Logger) -> Result<Interfaces> {
        let progress_chunk = value_t!(args, "progress-chunk", u32)
            .with_context(|_| ErrorKind::Config("progress-chunk is not a positive integer"))?;
        if progress_chunk == 0 {
            return Err(ErrorKind::Config("progress-chunck must be grater then 0").into());
        }
        Ok(Interfaces {
            logger,
            progress: !args.is_present("no-progress"),
            progress_chunk,
        })
    }

    /// Access the logger instnace.
    pub fn logger(&self) -> &Logger {
        &self.logger
    }

    /// Instantiate a new progress tracker.
    ///
    /// The `ProgressTracker` emits the given message to the given logger.
    /// The message is emitted only once every `process_chunk`.
    ///
    /// No progress will be outputted if the `--no-progress` command line argument is passed.
    pub fn progress<S>(&self, message: S) -> ProgressTracker
    where
        S: Into<String>,
    {
        ProgressTracker::new(
            self.progress_chunk,
            !self.progress,
            self.logger.clone(),
            message.into(),
        )
    }
}

/// Track progress of long running operations and emit logs about it.
pub struct ProgressTracker {
    chunk: u32,
    hidden: bool,
    logger: Logger,
    message: String,
    state: u32,
}

impl ProgressTracker {
    pub fn new(chunk: u32, hidden: bool, logger: Logger, message: String) -> ProgressTracker {
        ProgressTracker {
            chunk,
            hidden,
            logger,
            message,
            state: chunk,
        }
    }

    pub fn track(&mut self) {
        if self.hidden {
            return;
        }
        self.state -= 1;
        if self.state == 0 {
            self.state = self.chunk;
            info!(self.logger, "{}", self.message; "chunk" => self.chunk);
        }
    }
}
