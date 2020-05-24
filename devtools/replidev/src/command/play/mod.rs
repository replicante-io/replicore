use anyhow::Result;
use structopt::StructOpt;

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
#[derive(Debug, StructOpt)]
pub enum Opt {
    /// Delete persistent data for all nodes in clusters.
    #[structopt(name = "cluster-clean")]
    ClusterClean(CleanClusterOpt),

    /// Stop all playground nodes for clusters.
    #[structopt(name = "cluster-stop")]
    ClusterStop(StopClusterOpt),

    /// Delete persistent data a specific node.
    #[structopt(name = "node-clean")]
    NodeClean(CleanNodeOpt),

    /// Delete persistend data for all nodes.
    #[structopt(name = "node-clean-all")]
    NodeCleanAll(CleanCommonOpt),

    /// List all playground nodes.
    #[structopt(name = "node-list")]
    NodeList,

    /// Start a new playground node.
    #[structopt(name = "node-start")]
    NodeStart(StartNodeOpt),

    /// Stop a playground node.
    #[structopt(name = "node-stop")]
    NodeStop(StopNodeOpt),

    /// Clean the playground Replicante Core stack persistent data.
    #[structopt(name = "replicore-clean")]
    ReplicoreClean(CleanCommonOpt),

    /// Start the playground Replicante Core stack.
    #[structopt(name = "replicore-start")]
    ReplicoreStart,

    /// Stop the playground Replicante Core stack.
    #[structopt(name = "replicore-stop")]
    ReplicoreStop,

    /// Run the playground HTTP server (to simulate integrations for Replicante Core).
    #[structopt(name = "server")]
    Server,
}

impl Opt {
    pub fn need_actix_rt(&self) -> bool {
        match self {
            Self::Server => true,
            _ => false,
        }
    }
}

#[derive(Debug, StructOpt)]
pub struct CleanClusterOpt {
    /// List of clusters to clean up.
    #[structopt(name = "CLUSTER", required = true)]
    pub clusters: Vec<String>,

    #[structopt(flatten)]
    pub common: CleanCommonOpt,
}

#[derive(Debug, StructOpt)]
pub struct CleanCommonOpt {
    /// Confirm deleting the data.
    #[structopt(long)]
    pub confirm: bool,
}

#[derive(Debug, StructOpt)]
pub struct CleanNodeOpt {
    /// Cluster for the node to clean up.
    #[structopt(name = "CLUSTER", required = true)]
    pub cluster: String,

    /// List of nodes to clean up.
    #[structopt(name = "NODE", required = true)]
    pub nodes: Vec<String>,

    #[structopt(flatten)]
    pub common: CleanCommonOpt,
}

#[derive(Debug, StructOpt)]
pub struct StartNodeOpt {
    /// ID of the cluster to place the node into.
    #[structopt(name = "cluster-id", long)]
    pub cluster_id: Option<String>,

    /// Name to use for the new node.
    #[structopt(name = "node-name", long)]
    pub node_name: Option<String>,

    /// Store node to start.
    #[structopt(name = "STORE", required = true)]
    pub store: String,

    /// Add JSON files as extra variables passed to the command line.
    #[structopt(name = "var-file", long)]
    pub var_files: Vec<String>,

    /// Add extra variables passed to the command line.
    #[structopt(name = "var", long)]
    pub vars: Vec<String>,
}

#[derive(Debug, StructOpt)]
pub struct StopClusterOpt {
    /// List of clusters to stop.
    #[structopt(name = "CLUSTER", required = true)]
    pub clusters: Vec<String>,
}

#[derive(Debug, StructOpt)]
pub struct StopNodeOpt {
    /// List of nodes to stop.
    #[structopt(name = "NODE", required = true)]
    pub nodes: Vec<String>,
}

/// Manage Replicante Playground nodes.
pub async fn run(args: Opt, conf: Conf) -> Result<i32> {
    if !conf.project.allow_play() {
        anyhow::bail!(InvalidProject::new(conf.project, "play"));
    }
    let result = match args {
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
    };
    result.map_err(crate::error::wrap_for_anyhow)
}
