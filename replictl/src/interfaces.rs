use std::io;
use std::io::Write;

use indicatif::ProgressBar;
use clap::ArgMatches;
use slog::Logger;

use super::Result;


/// A container sturcture to inject dependencies.
pub struct Interfaces {
    logger: Logger,
    prompt: Prompt,

    // Internal attributes.
    progress: bool,
}

impl Interfaces {
    /// Create a new `Interfaces` container.
    pub fn new<'a>(args: &ArgMatches<'a>, logger: Logger) -> Interfaces {
        let prompt = Prompt {
            _logger: logger.clone()
        };
        Interfaces {
            logger,
            prompt,
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

    /// Access the user prompts interface.
    pub fn prompt(&self) -> &Prompt {
        &self.prompt
    }
}


/// Interface to interact with users over stdout/stdin.
pub struct Prompt {
    _logger: Logger,
}

impl Prompt {
    /// Ask the user for confirmation before something potentially harmful is done.
    pub fn confirm_danger(&self, prompt: &str) -> Result<bool> {
        print!("{} [y/N] ", prompt);
        io::stdout().flush()?;
        let mut reply = String::new();
        io::stdin().read_line(&mut reply)?;
        match reply.trim() {
            "y" => Ok(true),
            "yes" => Ok(true),
            _ => Ok(false),
        }
    }
}
