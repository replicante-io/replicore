use clap::ArgMatches;
use clap::Command;
use failure::Fail;
use slog::info;

use replicante_store_primary::Cursor;
use replicante_store_primary::ErrorKind as StoreErrorKind;
use replicante_util_failure::format_fail;

pub const COMMAND: &str = "primary-store-data";
const MODEL_ACTION: &str = "Action";
const MODEL_AGENT: &str = "Agent";
const MODEL_AGENT_INFO: &str = "AgentInfo";
const MODEL_CLUSTER_META: &str = "ClusterMeta";
const MODEL_CLUSTER_DISCOVERY: &str = "ClusterDiscovery";
const MODEL_NODE: &str = "Node";
const MODEL_SHARD: &str = "Shard";

use crate::outcome::Error;
use crate::outcome::Outcomes;
use crate::utils::primary_store_admin;
use crate::Interfaces;
use crate::Result;

macro_rules! scan_model {
    (
        $logger:ident,
        $interfaces:ident,
        $outcomes:ident,
        $model:ident,
        $cursor:expr $(,)?
    ) => {
        info!($logger, "Checking records for the '{}' model", $model);
        scan_collection($cursor, $model, &mut $outcomes, $interfaces);
        $outcomes.report(&$logger);
    };
}

pub fn command() -> Command {
    Command::new(COMMAND).about("Validate all records in the primary store")
}

pub fn run(args: &ArgMatches, interfaces: &Interfaces) -> Result<Outcomes> {
    let logger = interfaces.logger();
    let admin = primary_store_admin(args, logger.clone())?;
    let mut outcomes = Outcomes::new();

    info!(logger, "Validating all primary store records");
    scan_model!(
        logger,
        interfaces,
        outcomes,
        MODEL_ACTION,
        admin.data().actions(),
    );
    scan_model!(
        logger,
        interfaces,
        outcomes,
        MODEL_AGENT,
        admin.data().agents(),
    );
    scan_model!(
        logger,
        interfaces,
        outcomes,
        MODEL_AGENT_INFO,
        admin.data().agents_info(),
    );
    scan_model!(
        logger,
        interfaces,
        outcomes,
        MODEL_CLUSTER_META,
        admin.data().clusters_meta(),
    );
    scan_model!(
        logger,
        interfaces,
        outcomes,
        MODEL_CLUSTER_DISCOVERY,
        admin.data().cluster_discoveries(),
    );
    scan_model!(
        logger,
        interfaces,
        outcomes,
        MODEL_NODE,
        admin.data().nodes(),
    );
    scan_model!(
        logger,
        interfaces,
        outcomes,
        MODEL_SHARD,
        admin.data().shards(),
    );

    Ok(outcomes)
}

fn scan_collection<M>(
    cursor: replicante_store_primary::Result<Cursor<M>>,
    collection: &str,
    outcomes: &mut Outcomes,
    interfaces: &Interfaces,
) where
    M: ::std::fmt::Debug,
{
    let cursor = match cursor {
        Ok(cursor) => cursor,
        Err(error) => {
            let error = error.to_string();
            outcomes.error(Error::Generic(error));
            return;
        }
    };
    let mut tracker = interfaces.progress(format!("Scanned more {} documents", collection));
    for item in cursor {
        if let Err(error) = item {
            match error.kind() {
                StoreErrorKind::InvalidRecord(ref id) => {
                    let cause =
                        format_fail(error.cause().expect(
                            "primary store ErrorKind::InvalidRecord error must have a cause",
                        ));
                    outcomes.error(Error::UnableToParseModel(
                        collection.to_string(),
                        id.to_string(),
                        cause,
                    ));
                }
                _ => {
                    let error = format_fail(&error);
                    outcomes.error(Error::Generic(error));
                }
            }
        }
        tracker.track();
    }
}
