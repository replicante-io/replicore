use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use failure::ResultExt;
use failure::err_msg;
use slog::Logger;

use replicante::Config;
use replicante::ReplicanteQueues;
use replicante::task_payload::ClusterRefreshPayload;
use replicante_tasks::TaskQueue;
use replicante_tasks::admin::TasksAdmin;

use super::super::super::ErrorKind;
use super::super::super::Interfaces;
use super::super::super::Result;

use super::super::super::outcome::Error;
use super::super::super::outcome::Outcomes;


pub const COMMAND: &str = "tasks";
const COMMAND_DATA: &str = "data";


/// Check the `ReplicanteQueues::ClusterRefresh` tasks.
fn check_cluster_refresh(
    logger: &Logger, interfaces: &Interfaces, outcomes: &mut Outcomes,
    tasks: &TasksAdmin<ReplicanteQueues>
) -> Result<()> {
    info!(logger, "Checking tasks queue ..."; "queue" => ReplicanteQueues::ClusterRefresh.name());
    let mut tracker = interfaces.progress("Processed more tasks");
    let iter = tasks.scan(ReplicanteQueues::ClusterRefresh)?;
    for task in iter {
        match task {
            Err(error) => {
                let error = error.to_string();
                outcomes.error(Error::GenericError(error));
            },
            Ok(task) => {
                if let Err(error) = task.deserialize::<ClusterRefreshPayload>() {
                    outcomes.error(Error::UnableToParseModel(
                        "ClusterRefreshPayload".into(), task.id().to_string(), error.to_string()
                    ));
                }
            },
        }
        tracker.track();
    }
    outcomes.report(logger);
    Ok(())
}


/// Configure the `replictl check tasks` command parser.
pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("Check commands for the tasks subsystem")
        .subcommand(
            SubCommand::with_name(COMMAND_DATA)
            .about("Check all tasks for format incompatibilities")
        )
}


/// Check ALL tasks in the messaging platform for compatibility with this version of replicante.
///
/// The following checks are performed:
///
///   * Each task is fetched and parsed.
///   * Each task payload is parsed.
pub fn data<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let logger = interfaces.logger();
    info!(logger, "Checking tasks data");
    let confirm = interfaces.prompt().confirm_danger(
        "About to scan ALL tasks in the messaging platform. \
        This could impact your production system. \
        Would you like to proceed?"
    )?;
    if !confirm {
        error!(logger, "Cannot check without user confirmation");
        return Err(ErrorKind::Legacy(err_msg("operation aborded by the user")).into());
    }

    let mut outcomes = Outcomes::new();
    let config = args.value_of("config").unwrap();
    let config = Config::from_file(config)
        .context(ErrorKind::Legacy(err_msg("failed to check tasks")))?;
    let tasks: TasksAdmin<ReplicanteQueues> = TasksAdmin::new(logger.clone(), config.tasks)?;

    // Check all queues now.
    check_cluster_refresh(&logger, interfaces, &mut outcomes, &tasks)?;

    // Report results.
    if outcomes.has_errors() {
        error!(logger, "Tasks data checks failed");
        return Err(ErrorKind::Legacy(err_msg("tasks data checks failed")).into());
    }
    if outcomes.has_warnings() {
        warn!(logger, "Tasks data checks passed with warnings");
        return Ok(());
    }
    info!(logger, "Tasks data checks passed");
    Ok(())
}


/// Check commands for the tasks subsystem
pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let command = args.subcommand_matches(super::COMMAND).unwrap();
    let command = command.subcommand_matches(COMMAND).unwrap();
    let command = command.subcommand_name();
    match command {
        Some(COMMAND_DATA) => data(args, interfaces),
        None => Err(ErrorKind::Legacy(err_msg("need a tasks check to run")).into()),
        _ => Err(ErrorKind::Legacy(err_msg("received unrecognised command")).into()),
    }
}
