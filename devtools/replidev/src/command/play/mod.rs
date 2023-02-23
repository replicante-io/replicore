use anyhow::Result;
use clap::Args;
use clap::Subcommand;

use crate::conf::Conf;
use crate::error::InvalidProject;

mod cluster_clean;
mod cluster_stop;
mod node_clean;
mod node_clean_all;
mod node_list;
mod node_start;
mod node_stop;
mod replicore;
mod server;

/// Manage Replicante Playground nodes.
#[derive(Debug, Subcommand)]
pub enum Opt {
    /// Delete persistent data for all nodes in clusters.
    #[command(name = "cluster-clean")]
    ClusterClean(CleanClusterOpt),

    /// Stop all playground nodes for clusters.
    #[command(name = "cluster-stop")]
    ClusterStop(StopClusterOpt),

    /// Delete persistent data a specific node.
    #[command(name = "node-clean")]
    NodeClean(CleanNodeOpt),

    /// Delete persistend data for all nodes.
    #[command(name = "node-clean-all")]
    NodeCleanAll(CleanCommonOpt),

    /// List all playground nodes.
    #[command(name = "node-list")]
    NodeList,

    /// Start a new playground node.
    #[command(name = "node-start")]
    NodeStart(StartNodeOpt),

    /// Stop a playground node.
    #[command(name = "node-stop")]
    NodeStop(StopNodeOpt),

    /// Clean the playground Replicante Core stack persistent data.
    #[command(name = "replicore-clean")]
    ReplicoreClean(CleanCommonOpt),

    /// Start the playground Replicante Core stack.
    #[command(name = "replicore-start")]
    ReplicoreStart,

    /// Stop the playground Replicante Core stack.
    #[command(name = "replicore-stop")]
    ReplicoreStop,

    /// Run the playground HTTP server (to simulate integrations for Replicante Core).
    #[command(name = "server")]
    Server,
}

#[derive(Args, Debug)]
pub struct CleanClusterOpt {
    /// List of clusters to clean up.
    #[arg(name = "CLUSTER", required = true)]
    pub clusters: Vec<String>,

    #[command(flatten)]
    pub common: CleanCommonOpt,
}

#[derive(Args, Debug)]
pub struct CleanCommonOpt {
    /// Confirm deleting the data.
    #[arg(long)]
    pub confirm: bool,
}

#[derive(Args, Debug)]
pub struct CleanNodeOpt {
    /// Cluster for the node to clean up.
    #[arg(name = "CLUSTER", required = true)]
    pub cluster: String,

    /// List of nodes to clean up.
    #[arg(name = "NODE", required = true)]
    pub nodes: Vec<String>,

    #[command(flatten)]
    pub common: CleanCommonOpt,
}

#[derive(Args, Debug)]
pub struct StartNodeOpt {
    /// ID of the cluster to place the node into.
    #[arg(name = "cluster-id", long)]
    pub cluster_id: Option<String>,

    /// Name to use for the new node.
    #[arg(name = "node-name", long)]
    pub node_name: Option<String>,

    /// Store node to start.
    #[arg(name = "STORE", required = true)]
    pub store: String,

    /// Add JSON files as extra variables passed to the command line.
    #[arg(name = "var-file", long)]
    pub var_files: Vec<String>,

    /// Add extra variables passed to the command line.
    #[arg(name = "var", long)]
    pub vars: Vec<String>,
}

#[derive(Args, Debug)]
pub struct StopClusterOpt {
    /// List of clusters to stop.
    #[arg(name = "CLUSTER", required = true)]
    pub clusters: Vec<String>,
}

#[derive(Args, Debug)]
pub struct StopNodeOpt {
    /// List of nodes to stop.
    #[arg(name = "NODE", required = true)]
    pub nodes: Vec<String>,
}

/// Manage Replicante Playground nodes.
pub async fn run(args: Opt, conf: Conf) -> Result<i32> {
    if !conf.project.allow_play() {
        anyhow::bail!(InvalidProject::new(conf.project, "play"));
    }
    match args {
        Opt::ClusterClean(clean) => cluster_clean::run(&clean, &conf).await,
        Opt::ClusterStop(stop) => cluster_stop::run(&stop, &conf).await,
        Opt::NodeClean(clean) => node_clean::run(&clean, &conf).await,
        Opt::NodeCleanAll(clean) => node_clean_all::run(&clean, &conf).await,
        Opt::NodeList => node_list::run(&conf).await,
        Opt::NodeStart(start) => node_start::run(&start, &conf).await,
        Opt::NodeStop(stop) => node_stop::run(&stop, &conf).await,
        Opt::ReplicoreClean(clean) => replicore::clean(&clean, &conf).await,
        Opt::ReplicoreStart => replicore::start(&conf).await,
        Opt::ReplicoreStop => replicore::stop(&conf).await,
        Opt::Server => server::run(conf).await,
    }
}
