use clap::App;
use clap::ArgMatches;
use clap::SubCommand;


mod config;

use super::super::Interfaces;
use super::super::Result;


pub const COMMAND: &'static str = "check";
const DEEP_COMMAND: &'static str = "deep";
const QUICK_COMMAND: &'static str = "quick";
const UPDATE_COMMAND: &'static str = "update";


/// Configure the `replictl check` command parser.
pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("Perform checks on the system to find issues")
        .subcommand(config::command())
        .subcommand(SubCommand::with_name(DEEP_COMMAND)
            .about("Run all checks INCLUDING the ones that iterate over ALL data")
        )
        .subcommand(SubCommand::with_name(QUICK_COMMAND)
            .about("Run all checks that do NOT iterate over data (default command)")
        )
        .subcommand(SubCommand::with_name(UPDATE_COMMAND)
            .about("Run all checks to confirm an update is possible (iterates over ALL data!)")
        )
}


/// Switch the control flow to the requested check command.
pub fn run<'a>(args: ArgMatches<'a>, interfaces: Interfaces) -> Result<()> {
    match args.subcommand_matches(COMMAND).unwrap().subcommand_name() {
        Some(config::COMMAND) => config::run(args, interfaces),
        Some(DEEP_COMMAND) => run_deep(args, interfaces),
        Some(QUICK_COMMAND) => run_quick(args, interfaces),
        Some(UPDATE_COMMAND) => run_deep(args, interfaces),
        None => run_quick(args, interfaces),
        _ => Err("Received unrecognised command".into()),
    }
}


/// Run all checks INCLUDING the ones that iterate over ALL data.
fn run_deep<'a>(args: ArgMatches<'a>, interfaces: Interfaces) -> Result<()> {
    config::run(args, interfaces)
}


/// Run all checks that do NOT iterate over data.
fn run_quick<'a>(args: ArgMatches<'a>, interfaces: Interfaces) -> Result<()> {
    config::run(args, interfaces)
}
