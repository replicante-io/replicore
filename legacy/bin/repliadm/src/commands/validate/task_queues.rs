use clap::ArgMatches;
use clap::Command;
use failure::ResultExt;
use slog::info;

use replicante_service_tasks::admin::TasksAdmin;
use replicante_service_tasks::TaskQueue;
use replicante_util_failure::format_fail;

use replicore_models_tasks::ReplicanteQueues;

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
                    $outcomes.error(Error::Generic(error));
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

pub fn command() -> Command {
    Command::new(COMMAND).about("Validate tasks waiting to be processed or re-tried")
}

pub fn run(args: &ArgMatches, interfaces: &Interfaces) -> Result<Outcomes> {
    let config = load_config(args)?;
    let logger = interfaces.logger();
    let tasks: TasksAdmin<ReplicanteQueues> = TasksAdmin::new(logger.clone(), config.tasks)
        .with_context(|_| ErrorKind::AdminInit("tasks"))?;
    let mut outcomes = Outcomes::new();

    scan_queue!(
        logger,
        interfaces,
        outcomes,
        tasks,
        ReplicanteQueues::DiscoverClusters,
        replicore_models_tasks::payload::DiscoverClustersPayload,
    );
    scan_queue!(
        logger,
        interfaces,
        outcomes,
        tasks,
        ReplicanteQueues::OrchestrateCluster,
        replicore_models_tasks::payload::OrchestrateClusterPayload,
    );

    outcomes.report(logger);
    Ok(outcomes)
}
