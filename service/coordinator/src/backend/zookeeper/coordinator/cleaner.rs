use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use failure::ResultExt;
use humthreads::Builder;
use humthreads::Thread;
use humthreads::ThreadScope;
use slog::debug;
use slog::info;
use slog::Logger;
use zookeeper::ZkError;

use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;

use super::super::super::super::config::ZookeeperConfig;
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

use super::super::constants::PREFIX_ELECTION;
use super::super::constants::PREFIX_LOCK;
use super::super::constants::PREFIX_NODE;
use super::super::metrics::ZOO_CLEANUP_COUNT;
use super::super::metrics::ZOO_OP_DURATION;
use super::super::metrics::ZOO_OP_ERRORS_COUNT;
use super::super::metrics::ZOO_TIMEOUTS_COUNT;
use super::election::ZookeeperElection;
use super::Client;

/// Background thread to cleanup unused nodes.
///
/// Prevent the prefix nodes that do not contain anything from piling up without value.
/// Once the new container znode type is stable this code can be dropped in favour of that.
pub struct Cleaner {
    handle: Mutex<Option<Thread<()>>>,
    logger: Logger,
    shutdown_signal: Option<ShutdownSender>,
}

impl Cleaner {
    pub fn new(
        client: Arc<Client>,
        config: ZookeeperConfig,
        node_id: NodeId,
        logger: Logger,
    ) -> Result<Cleaner> {
        let (sender, receiver) = LoopingElectionOpts::shutdown_channel();
        let inner_logger = logger.clone();
        let handle = Builder::new("r:s:coordinator:zoo:c")
            .full_name("replicore:service:coordinator:zookeeper:cleaner")
            .spawn(move |scope| {
                scope.activity("initialising zookeeper cleaner election");
                let logger = inner_logger;
                let cleaner = InnerCleaner {
                    cleanup_limit: config.cleanup.limit,
                    client: Arc::clone(&client),
                    logger: logger.clone(),
                    thread: scope,
                };
                let id = "zookeeper-cleaner";
                let election = Election::new(
                    id.to_string(),
                    Box::new(ZookeeperElection::new(client, id, node_id, logger.clone())),
                );
                let opts = LoopingElectionOpts::new(election, cleaner)
                    .loop_delay(Duration::from_secs(config.cleanup.interval))
                    .shutdown_receiver(receiver);
                let opts = match config.cleanup.term {
                    0 => opts,
                    term => opts.election_term(term),
                };
                let mut election = LoopingElection::new(opts, logger);
                election.loop_forever();
            })
            .with_context(|_| ErrorKind::SpawnThread("zookeeper cleaner"))?;
        Ok(Cleaner {
            handle: Mutex::new(Some(handle)),
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
        let handle = self
            .handle
            .lock()
            .expect("zookeeper cleaner thread lock poisoned")
            .take();
        if let Some(handle) = handle {
            if let Err(error) = handle.join() {
                capture_fail!(
                    &error,
                    self.logger,
                    "Zookeeper cleaner thread paniced";
                    failure_info(&error),
                );
            }
        }
    }
}

/// Helper class to collect worker thread context.
struct InnerCleaner {
    cleanup_limit: usize,
    client: Arc<Client>,
    logger: Logger,
    thread: ThreadScope,
}

impl InnerCleaner {
    /// Clean children of the given path.
    fn clean(&self, path: &str, limit: usize) -> Result<usize> {
        let client = self.client.get()?;
        let mut limit = limit;
        let children = Client::get_children(&client, path, false)
            .with_context(|_| ErrorKind::Backend("children lookup"))?;
        for child in children {
            let child = format!("{}/{}", path, child);
            let timer = ZOO_OP_DURATION.with_label_values(&["exists"]).start_timer();
            let stats = match client.exists(&child, false) {
                Err(ZkError::NoNode) | Ok(None) => {
                    timer.observe_duration();
                    continue;
                }
                Err(error) => {
                    timer.observe_duration();
                    ZOO_OP_ERRORS_COUNT.with_label_values(&["exists"]).inc();
                    if error == ZkError::OperationTimeout {
                        ZOO_TIMEOUTS_COUNT.inc();
                    }
                    return Err(error).with_context(|_| ErrorKind::Backend("node lookup"))?;
                }
                Ok(Some(stats)) => stats,
            };
            timer.observe_duration();

            // Look only at empty nodes.
            if stats.num_children != 0 {
                continue;
            }

            // Delete and count.
            match Client::delete(&client, &child, Some(stats.version), None, None) {
                Err(ZkError::NoNode) | Err(ZkError::NotEmpty) | Ok(()) => (),
                Err(error) => {
                    return Err(error).with_context(|_| ErrorKind::Backend("node delete"))?;
                }
            };
            ZOO_CLEANUP_COUNT.inc();
            limit -= 1;
            if limit == 0 {
                return Ok(0);
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
        capture_fail!(
            &error,
            self.logger,
            "Zookeeper background cleaner election error";
            failure_info(&error)
        );
        LoopingElectionControl::Continue
    }

    fn post_check(&self, election: &Election) -> Result<LoopingElectionControl> {
        self.thread
            .activity(format!("(idle) election status: {:?}", election.status()));
        Ok(LoopingElectionControl::Proceed)
    }

    fn pre_check(&self, election: &Election) -> Result<LoopingElectionControl> {
        self.thread
            .activity(format!("election status: {:?}", election.status()));
        Ok(LoopingElectionControl::Proceed)
    }

    fn primary(&self, _: &Election) -> Result<LoopingElectionControl> {
        info!(self.logger, "Running zookeeper cleanup cycle");
        let _activity = self
            .thread
            .scoped_activity("cleaning empty zookeeper znodes");
        if let Err(error) = self.cycle() {
            capture_fail!(
                &error,
                self.logger,
                "Zookeeper cleanup cycle failed";
                failure_info(&error)
            );
        }
        debug!(self.logger, "Zookeeper cleanup cycle ended");
        Ok(LoopingElectionControl::Proceed)
    }

    fn secondary(&self, _: &Election) -> Result<LoopingElectionControl> {
        debug!(self.logger, "Zookeeper background cleaner is secondary");
        Ok(LoopingElectionControl::Proceed)
    }
}
