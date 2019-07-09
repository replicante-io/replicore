use clap::App;
use clap::ArgMatches;
use clap::SubCommand;

use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

mod primary;
mod view;

pub const COMMAND: &str = "stores";

/// Configure the `replictl check stores` command parser.
pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("Check stores for incompatibilities")
        .subcommand(primary::command())
        .subcommand(view::command())
}

/// Switch the control flow to the requested check command.
pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let command = args.subcommand_matches(super::COMMAND).unwrap();
    let command = command.subcommand_matches(COMMAND).unwrap();
    let command = command.subcommand_name();
    match command {
        Some(primary::COMMAND) => primary::run(args, interfaces),
        Some(view::COMMAND) => view::run(args, interfaces),
        None => Err(ErrorKind::NoCommand("replictl check stores").into()),
        Some(name) => {
            Err(ErrorKind::UnkownSubcommand("replictl check stores", name.to_string()).into())
        }
    }
}

/// Run all checks INCLUDING the ones that iterate over ALL data.
pub fn run_deep<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let pschema = primary::schema(args, interfaces);
    let pdata = primary::data(args, interfaces);
    let vschema = view::schema(args, interfaces);
    let vdata = view::data(args, interfaces);
    pschema?;
    pdata?;
    vschema?;
    vdata?;
    Ok(())
}

/// Run all checks that do NOT iterate over data.
pub fn run_quick<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let pschema = primary::schema(args, interfaces);
    let vschema = view::schema(args, interfaces);
    pschema?;
    vschema?;
    Ok(())
}
