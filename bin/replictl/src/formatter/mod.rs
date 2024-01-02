//! Abstract how information is presented to users to enable different interaction styles.
//!
//! For example:
//!
//! - The default `Human` formatter aims to provide output suitable for an interactive session
//!   where people issue commands and review results.
//! - The `JSON` formatter aims to provide output suitable for an automated script.
use anyhow::Result;
use clap::Args;
use clap::ValueEnum;

use replisdk::core::models::api::NamespaceEntry;
use replisdk::core::models::api::PlatformEntry;

//mod json;
mod human;

pub mod ops;

use crate::context::Context;
use crate::globals::Globals;

/// Present a list of [`Context`]s to the user.
pub trait ContextList {
    /// Append a new context into the list being formatted.
    fn append(&mut self, name: &str, context: &Context, active: bool) -> Result<()>;

    /// Handle the now complete list of contexts and emit it to standard output.
    fn finish(&mut self) -> Result<()>;
}

/// List of available output formats.
#[derive(Copy, Clone, Debug, Default, ValueEnum)]
pub enum FormatId {
    /// Optimise output for viewing by humans.
    #[default]
    Human,

    /// Output information as JSON documents.
    Json,
}

/// Configure output formatting for `replictl`.
#[derive(Args, Debug)]
pub struct FormatOpts {
    /// Select the format to use for output.
    #[arg(
        long = "format",
        global = true,
        env = "RCTL_FORMAT",
        default_value_t,
        value_enum
    )]
    pub format: FormatId,
}

/// Present information to users in their preferred format.
pub struct Formatter {
    /// Runtime strategy to execute formatting operations with.
    strategy: Box<dyn FormatterStrategy>,
}

impl Formatter {
    /// Execute the specified formatting operation.
    pub fn format<O>(&self, globals: &Globals, op: O) -> O::Response
    where
        O: self::ops::FormatOp,
    {
        let op = op.into();
        let result = self.strategy.format(globals, op);
        O::Response::from(result)
    }
}

/// Interface to implement user output formatting.
pub trait FormatterStrategy {
    /// Execute the requested formatting operation.
    fn format(&self, globals: &Globals, op: self::ops::Ops) -> self::ops::Responses;
}

/// Present a list of [`NamespaceEntry`]s to the user.
pub trait NamespaceList {
    /// Append a new namespace entry into the list being formatted.
    fn append(&mut self, entry: &NamespaceEntry) -> Result<()>;

    /// Handle the now complete list of namespace entries and emit it to standard output.
    fn finish(&mut self) -> Result<()>;
}

/// Present a list of [`PlatformEntry`]s to the user.
pub trait PlatformList {
    /// Append a new platform entry into the list being formatted.
    fn append(&mut self, entry: &PlatformEntry) -> Result<()>;

    /// Handle the now complete list of platform entries and emit it to standard output.
    fn finish(&mut self) -> Result<()>;
}

/// Instantiate a formatter based on CLI configuration.
pub fn select(format: &FormatOpts) -> Formatter {
    let strategy = match format.format {
        FormatId::Human => Box::new(self::human::HumanFormatter),
        FormatId::Json => todo!("JSON format not yet supported"),
    };
    Formatter { strategy }
}
