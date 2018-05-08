use indicatif::ProgressBar;
use clap::ArgMatches;
use slog::Logger;


/// A container sturcture to inject dependencies.
pub struct Interfaces {
    logger: Logger,

    // Internal attributes.
    progress: bool,
}

impl Interfaces {
    /// Create a new `Interfaces` container.
    pub fn new<'a>(args: &ArgMatches<'a>, logger: Logger) -> Interfaces {
        Interfaces {
            logger,
            progress: !args.is_present("no-progress"),
        }
    }

    /// Access the logger instnace.
    pub fn logger(&self) -> &Logger {
        &self.logger
    }

    /// Instantiate a new progress bar.
    ///
    /// An optional length parameter determines the max value.
    /// If the length is not specified a spinning progress bar is returned.
    ///
    /// The progress bar will not render if the `--no-progress` command line argument is passed.
    pub fn progress(&self, len: Option<u64>) -> ProgressBar {
        if !self.progress {
            return ProgressBar::hidden();
        }
        match len {
            Some(len) => ProgressBar::new(len),
            None => ProgressBar::new_spinner(),
        }
    }
}
