use clap::ArgMatches;
use clap::Command;
use slog::info;

use replicante_store_view::Cursor;
use replicante_util_failure::format_fail;

pub const COMMAND: &str = "view-store-data";
const MODEL_ACTION: &str = "Action";
const MODEL_ACTION_HISTORY: &str = "ActionHistory";
const MODEL_EVENT: &str = "Event";

use crate::outcome::Error;
use crate::outcome::Outcomes;
use crate::utils::view_store_admin;
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
    Command::new(COMMAND).about("Validate all records in the view store")
}

pub fn run(args: &ArgMatches, interfaces: &Interfaces) -> Result<Outcomes> {
    let logger = interfaces.logger();
    let admin = view_store_admin(args, logger.clone())?;
    let mut outcomes = Outcomes::new();

    info!(logger, "Validating all view store records");
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
        MODEL_ACTION_HISTORY,
        admin.data().actions_history(),
    );
    scan_model!(
        logger,
        interfaces,
        outcomes,
        MODEL_EVENT,
        admin.data().events(),
    );

    Ok(outcomes)
}

fn scan_collection<M>(
    cursor: replicante_store_view::Result<Cursor<M>>,
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
            let error = format_fail(&error);
            outcomes.error(Error::Generic(error));
        }
        tracker.track();
    }
}
