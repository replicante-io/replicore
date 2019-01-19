use std::sync::Arc;
use std::thread::Builder;
use std::thread::JoinHandle;
use std::time::Duration;

use failure::ResultExt;
use slog::Logger;
use zookeeper::ZkError;

use replicante_util_failure::failure_info;

use super::super::super::super::Election;
use super::super::super::super::Error;
use super::super::super::super::ErrorKind;
use super::super::super::super::LoopingElection;
use super::super::super::super::LoopingElectionControl;
use super::super::super::super::LoopingElectionLogic;
use super::super::super::super::LoopingElectionOpts;
use super::super::super::super::NodeId;
use super::super::super::super::Result;
use super::super::super::super::ShutdownSender;
use super::super::super::super::config::ZookeeperConfig;

use super::super::constants::PREFIX_ELECTION;
use super::super::constants::PREFIX_LOCK;
use super::super::constants::PREFIX_NODE;

use super::super::metrics::ZOO_CLEANUP_COUNT;
use super::super::metrics::ZOO_OP_DURATION;
use super::super::metrics::ZOO_OP_ERRORS_COUNT;
use super::super::metrics::ZOO_TIMEOUTS_COUNT;
use super::Client;
use super::election::ZookeeperElection;


/// Background thread to cleanup unused nodes.
///
/// Prevent the prefix nodes that do not contain anything from piling up without value.
/// Once the new container znode type is stable this code can be dropped in favour of that.
pub struct Cleaner {
    handle: Option<JoinHandle<()>>,
    logger: Logger,
    shutdown_signal: Option<ShutdownSender>,
}

impl Cleaner {
    pub fn new(
        client: Arc<Client>, config: ZookeeperConfig, node_id: NodeId, logger: Logger
    ) -> Result<Cleaner> {
        let (sender, receiver) = LoopingElectionOpts::shutdown_channel();
        let inner_logger = logger.clone();
        let handle = Builder::new().name("r:coordinator:zoo:cleaner".into()).spawn(move || {
            let logger = inner_logger;
            let cleaner = InnerCleaner {
                cleanup_limit: config.cleanup.limit,
                client: Arc::clone(&client),
                logger: logger.clone(),
            };
            let id = "zookeeper-cleaner".to_string();
            let election = Election::new(id.clone(), Box::new(ZookeeperElection::new(
                client, id, node_id, logger.clone()
            )));
            let opts = LoopingElectionOpts::new(election, cleaner)
                .loop_delay(Duration::from_secs(config.cleanup.interval))
                .shutdown_receiver(receiver);
            let opts = match config.cleanup.term {
                0 => opts,
                term => opts.election_term(term),
            };
            let mut election = LoopingElection::new(opts, logger);
            election.loop_forever();
        }).context(ErrorKind::SpawnThread("zookeeper cleaner"))?;
        Ok(Cleaner {
            handle: Some(handle),
            logger,
            shutdown_signal: Some(sender),
        })
    }
}

impl Drop for Cleaner {
    fn drop(&mut self) {
        if let Some(shutdown_signal) = self.shutdown_signal.take() {
            drop(shutdown_signal);
        }
        if let Some(handle) = self.handle.take() {
            if let Err(error) = handle.join() {
                error!(self.logger, "Zookeeper cleaner thread paniced"; "error" => ?error);
            }
        }
    }
}


/// Helper class to collect worker thread context.
struct InnerCleaner {
    client: Arc<Client>,
    cleanup_limit: usize,
    logger: Logger,
}

impl InnerCleaner {
    /// Clean children of the given path.
    fn clean(&self, path: &str, limit: usize) -> Result<usize> {
        let client = self.client.get()?;
        let mut limit = limit;

        let children = Client::get_children(&client, path, false)
            .context(ErrorKind::Backend("children lookup"))?;
        for child in children {
            let child = format!("{}/{}", path, child);
            let timer = ZOO_OP_DURATION.with_label_values(&["exists"]).start_timer();
            let stats = match client.exists(&child, false) {
                Err(ZkError::NoNode) | Ok(None) => {
                    timer.observe_duration();
                    continue;
                },
                Err(error) => {
                    timer.observe_duration();
                    ZOO_OP_ERRORS_COUNT.with_label_values(&["exists"]).inc();
                    if error == ZkError::OperationTimeout {
                        ZOO_TIMEOUTS_COUNT.inc();
                    }
                    return Err(error).context(ErrorKind::Backend("node lookup"))?;
                },
                Ok(Some(stats)) => stats,
            };
            timer.observe_duration();

            // Look only at empty nodes.
            if stats.num_children != 0 {
                continue;
            }

            // Delete and count.
            match Client::delete(&client, &child, Some(stats.version)) {
                Err(ZkError::NoNode) |
                    Err(ZkError::NotEmpty) |
                    Ok(()) => (),
                Err(error) => return Err(error).context(ErrorKind::Backend("node delete"))?,
            };
            ZOO_CLEANUP_COUNT.inc();
            limit = limit - 1;
            if limit == 0 {
                return Ok(0)
            }
        }

        Ok(limit)
    }

    /// Perform a single zookeeper cleanup cycle.
    fn cycle(&self) -> Result<()> {
        let limit = self.cleanup_limit;
        let limit = self.clean(PREFIX_ELECTION, limit)?;
        if self.cycle_limit(limit) {
            return Ok(());
        }
        let limit = self.clean(PREFIX_LOCK, limit)?;
        if self.cycle_limit(limit) {
            return Ok(());
        }
        let limit = self.clean(PREFIX_NODE, limit)?;
        if self.cycle_limit(limit) {
            return Ok(());
        }
        Ok(())
    }

    /// Check if the limit of deletes for this cycle has been reached.
    fn cycle_limit(&self, limit: usize) -> bool {
        if limit == 0 {
            info!(self.logger, "Reached limit of nodes to clean for one cycle");
            return true;
        }
        false
    }
}

impl LoopingElectionLogic for InnerCleaner {
    fn handle_error(&self, error: Error) -> LoopingElectionControl {
        error!(self.logger, "Zookeeper background cleaner election error"; failure_info(&error));
        LoopingElectionControl::Continue
    }

    fn primary(&self, _: &Election) -> Result<LoopingElectionControl> {
        info!(self.logger, "Running zookeeper cleanup cycle");
        if let Err(error) = self.cycle() {
            error!(self.logger, "Zookeeper cleanup cycle failed"; failure_info(&error));
        }
        debug!(self.logger, "Zookeeper cleanup cycle ended");
        Ok(LoopingElectionControl::Proceed)
    }

    fn secondary(&self, _: &Election) -> Result<LoopingElectionControl> {
        debug!(self.logger, "Zookeeper background cleaner is secondary");
        Ok(LoopingElectionControl::Proceed)
    }
}
