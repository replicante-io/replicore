use slog::Logger;

use crate::interfaces::Interfaces;

mod action_approve;
mod action_disapprove;
mod refresh;

/// Attach cluster-related core API handlers.
pub fn attach(logger: Logger, interfaces: &mut Interfaces) {
    action_approve::attach(logger.clone(), interfaces);
    action_disapprove::attach(logger.clone(), interfaces);
    refresh::attach(logger, interfaces);
}
