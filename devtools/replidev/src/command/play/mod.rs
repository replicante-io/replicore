use structopt::StructOpt;

use crate::conf::Conf;
use crate::ErrorKind;
use crate::Result;

mod node_list;
mod node_start;

/// Manage Replicante Playground nodes.
#[derive(Debug, StructOpt)]
pub enum CliOpt {
    /// List all playground nodes.
    #[structopt(name = "node-list")]
    NodeList,

    /// Start a new playground node.
    #[structopt(name = "node-start")]
    NodeStart(NodeOpt),
}

#[derive(Debug, StructOpt)]
pub struct NodeOpt {
    /// ID of the cluster to place the node into.
    #[structopt(name = "cluster-id", long)]
    cluster_id: Option<String>,

    /// Store node to start.
    #[structopt(name = "STORE", required = true)]
    store: String,
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
    }
}
