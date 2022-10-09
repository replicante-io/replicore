use anyhow::Result;
use clap::Args;
use clap::Subcommand;
use slog::Logger;
use uuid::Uuid;

mod approve;
mod disapprove;
mod node_list;
mod orchestrator_approve;
mod orchestrator_disapprove;
mod orchestrator_list;

/// Command line options common to all action commands.
///
/// This is included, possibly flattened, as arguments to leaf commands instead of additional
/// options at the `action` level because we want to ensure the command is specified before
/// these options.
///
/// In other words we want `replictl action {approve, ...} $ACTION_ID`
/// and not `replictl action $ACTION_ID {approve, ...}`.
#[derive(Args, Debug)]
pub struct CommonOpt {
    /// ID of the action to operate on.
    #[arg(env = "RCTL_ACTION")]
    pub action: Uuid,
}

/// Show and manage actions.
#[derive(Debug, Subcommand)]
pub enum Opt {
    /// Approve an action that is pending approval.
    #[command(alias = "approve")]
    ApproveNodeAction(CommonOpt),

    /// Approve an orchestrator action that is pending approval.
    ApproveOrchestratorAction(CommonOpt),

    /// Disapprove (reject) an action that is pending approval.
    #[command(alias = "disapprove")]
    DisapproveNodeAction(CommonOpt),

    /// Disapprove (reject) an orchestrator action that is pending approval.
    DisapproveOrchestratorAction(CommonOpt),

    /// List known node actions for a cluster.
    ListNodeActions,

    /// List known orchestrator actions for a cluster.
    ListOrchestratorActions,
}

/// Execute the selected command.
pub async fn execute(logger: &Logger, cli: &crate::Cli, action_cmd: &Opt) -> Result<i32> {
    match &action_cmd {
        Opt::ApproveNodeAction(approve_opt) => approve::execute(logger, cli, approve_opt).await,
        Opt::ApproveOrchestratorAction(approve_opt) => {
            orchestrator_approve::execute(logger, cli, approve_opt).await
        }
        Opt::DisapproveNodeAction(disapprove_opt) => {
            disapprove::execute(logger, cli, disapprove_opt).await
        }
        Opt::DisapproveOrchestratorAction(disapprove_opt) => {
            orchestrator_disapprove::execute(logger, cli, disapprove_opt).await
        }
        Opt::ListNodeActions => node_list::execute(logger, cli).await,
        Opt::ListOrchestratorActions => orchestrator_list::execute(logger, cli).await,
    }
}
