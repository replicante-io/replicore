use structopt::StructOpt;

use crate::conf::Conf;
use crate::ErrorKind;
use crate::Result;

mod node_list;
mod node_start;
mod replicore;

/// Manage Replicante Playground nodes.
#[derive(Debug, StructOpt)]
pub enum CliOpt {
    /// List all playground nodes.
    #[structopt(name = "node-list")]
    NodeList,

    /// Start a new playground node.
    #[structopt(name = "node-start")]
    NodeStart(StartNodeOpt),

    /// Clean the playground Replicante Core stack persistent data.
    #[structopt(name = "replicore-clean")]
    ReplicoreClean(CleanOpt),

    /// Start the playground Replicante Core stack.
    #[structopt(name = "replicore-start")]
    ReplicoreStart,

    /// Stop the playground Replicante Core stack.
    #[structopt(name = "replicore-stop")]
    ReplicoreStop,
}

#[derive(Debug, StructOpt)]
pub struct CleanOpt {
    /// Confirm deleting the data.
    #[structopt(long)]
    pub confirm: bool,
}

#[derive(Debug, StructOpt)]
pub struct StartNodeOpt {
    /// ID of the cluster to place the node into.
    #[structopt(name = "cluster-id", long)]
    cluster_id: Option<String>,

    /// Store node to start.
    #[structopt(name = "STORE", required = true)]
    store: String,

    /// Add JSON files as extra variables passed to the command line.
    #[structopt(name = "var-file", long)]
    var_files: Vec<String>,

    /// Add extra variables passed to the command line.
    #[structopt(name = "var", long)]
    vars: Vec<String>,
}

/// Manage Replicante Playground nodes.
pub fn run(args: CliOpt, conf: Conf) -> Result<bool> {
    if !conf.project.allow_play() {
        let error = ErrorKind::invalid_project(conf.project, "replidev play");
        return Err(error.into());
    }
    match args {
        CliOpt::NodeList => node_list::run(&conf),
        CliOpt::NodeStart(start) => node_start::run(&start, &conf),
        CliOpt::ReplicoreClean(clean) => replicore::clean(&clean, &conf),
        CliOpt::ReplicoreStart => replicore::start(&conf),
        CliOpt::ReplicoreStop => replicore::stop(&conf),
    }
}
