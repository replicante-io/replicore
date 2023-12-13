//! Container for data to made accessible to all `replictl` commands.
use slog::Logger;

use crate::formatter::Formatter;
use crate::Cli;

/// Container for data to made accessible to all `replictl` commands.
pub struct Globals {
    /// Parse CLI arguments.
    pub cli: Cli,

    /// Configured process formatter for all output.
    pub formatter: Formatter,

    /// Configured process logger for advanced users feedback/debugging.
    pub logger: Logger,
}

impl Globals {
    /// Initialise `replictl` process [`Globals`].
    pub async fn initialise(cli: Cli) -> Self {
        // TODO: init logging
        let logger = slog::Logger::root(slog::Discard, slog::o!());
        let formatter = crate::formatter::select(&cli.format);
        Globals {
            cli,
            formatter,
            logger,
        }
    }
}
