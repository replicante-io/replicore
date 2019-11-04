use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use failure::ResultExt;
use slog::info;

use replicante::task_payload::ClusterRefreshPayload;
use replicante::ReplicanteQueues;
use replicante_service_tasks::admin::TasksAdmin;
use replicante_service_tasks::TaskQueue;
use replicante_util_failure::format_fail;

pub const COMMAND: &str = "task-queues";

use crate::outcome::Error;
use crate::outcome::Outcomes;
use crate::utils::load_config;
use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

/// Scan the given task queue and validate payloads.
macro_rules! scan_queue {
    (
        $logger:ident,
        $interfaces:ident,
        $outcomes:ident,
        $tasks:ident,
        $queue:expr,
        $payload_model:ty $(,)?
    ) => {
        info!($logger, "Checking tasks queue ..."; "queue" => $queue.name());
        let mut tracker = $interfaces.progress("Processed more tasks");
        let iter = $tasks
            .scan($queue)
            .with_context(|_| ErrorKind::ValidationError("queued tasks"))?;
        for task in iter {
            match task {
                Err(error) => {
                    let error = format_fail(&error);
                    $outcomes.error(Error::GenericError(error));
                }
                Ok(task) => {
                    if let Err(error) = task.deserialize::<$payload_model>() {
                        $outcomes.error(Error::UnableToParseModel(
                            stringify!($payload_model).into(),
                            task.id().to_string(),
                            error.to_string(),
                        ));
                    }
                }
            }
            tracker.track();
        }
    };
}

pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("Validate tasks waiting to be processed or re-tried")
}

pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<Outcomes> {
    let config = load_config(args)?;
    let logger = interfaces.logger();
    let tasks: TasksAdmin<ReplicanteQueues> = TasksAdmin::new(logger.clone(), config.tasks)
        .with_context(|_| ErrorKind::AdminInit("tasks"))?;
    let mut outcomes = Outcomes::new();

    // Scan every queue here.
    scan_queue!(
        logger,
        interfaces,
        outcomes,
        tasks,
        ReplicanteQueues::ClusterRefresh,
        ClusterRefreshPayload,
    );
    outcomes.report(&logger);

    Ok(outcomes)
}
