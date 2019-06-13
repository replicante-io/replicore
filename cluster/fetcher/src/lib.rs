use std::sync::Arc;
use std::time::Duration;

use failure::ResultExt;
use opentracingrust::Log;
use opentracingrust::Span;
use opentracingrust::Tracer;
use slog::debug;
use slog::warn;
use slog::Logger;

use replicante_agent_client::HttpClient;
use replicante_models_core::Agent;
use replicante_models_core::AgentStatus;
use replicante_models_core::ClusterDiscovery;
use replicante_service_coordinator::NonBlockingLockWatcher;
use replicante_store_primary::store::Store;
use replicante_streams_events::EventsStream;
use replicante_util_failure::format_fail;

mod agent;
mod error;
mod metrics;
mod node;
mod shard;
mod snapshotter;

use self::agent::AgentFetcher;
use self::metrics::FETCHER_DURATION;
use self::metrics::FETCHER_ERRORS_COUNT;
use self::node::NodeFetcher;
use self::shard::ShardFetcher;

pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;
pub use self::metrics::register_metrics;
pub use self::snapshotter::Snapshotter;

struct ClusterIdentityChecker {
    display_name: Option<String>,
    id: String,
}

impl ClusterIdentityChecker {
    fn check_or_set_display_name(&mut self, display_name: &str, node_id: &str) -> Result<()> {
        if self.display_name.is_none() {
            self.display_name = Some(display_name.to_string());
            return Ok(());
        }
        let current = self.display_name.as_ref().unwrap();
        if current == display_name {
            return Ok(());
        }
        Err(ErrorKind::ClusterDisplayNameDoesNotMatch(
            current.to_string(),
            display_name.to_string(),
            node_id.to_string(),
        )
        .into())
    }

    fn check_id(&mut self, id: &str, node_id: &str) -> Result<()> {
        if self.id == id {
            return Ok(());
        }
        Err(
            ErrorKind::ClusterIdDoesNotMatch(self.id.clone(), id.to_string(), node_id.to_string())
                .into(),
        )
    }

    fn new(id: String, display_name: Option<String>) -> ClusterIdentityChecker {
        ClusterIdentityChecker { display_name, id }
    }
}

/// Node (agent and datastore) status fetching and processing logic.
///
/// The Fetcher is responsible for:
///
///   1. For each node:
///     1. Attempt to fetch agent info.
///     2. Persist `AgentInfo` record (if fetch succeeded).
///     3. Attempt to fetch node info (if agent is up).
///     4. Persist `Node` record (if fetch succeeded).
///     5. Attempt to fetch shards status (only if agent and datastore are up).
///     6. Persist each `Shard` record (if fetch succeeded).
///     7. Persist the `Agent` record.
pub struct Fetcher {
    agent: AgentFetcher,
    logger: Logger,
    node: NodeFetcher,
    shard: ShardFetcher,
    store: Store,
    timeout: Duration,
    tracer: Arc<Tracer>,
}

impl Fetcher {
    pub fn new(
        logger: Logger,
        events: EventsStream,
        store: Store,
        timeout: Duration,
        tracer: Arc<Tracer>,
    ) -> Fetcher {
        let agent = AgentFetcher::new(events.clone(), store.clone());
        let node = NodeFetcher::new(events.clone(), store.clone());
        let shard = ShardFetcher::new(events, store.clone());
        Fetcher {
            agent,
            logger,
            node,
            shard,
            store,
            timeout,
            tracer,
        }
    }

    /// Fetch an optimistic view of the cluster state.
    ///
    /// # Errors
    /// The frech process can encounter two kinds of errors:
    ///
    ///   * Core errors: store, coordinator, internal logic, ...
    ///   * Remote errors: agent is down, network issue, invalid data returned, ...
    ///
    /// Core errors are returned and interupt the fetching process early (if the primary store is
    /// failing to respond it is likely to fail again in a short time).
    ///
    /// Remote errors are logged and accounted for as part of the refresh process (a remote agent
    /// crashing should not prevent the full cluster from being checked).
    /// Refresh the optimistic view on a cluster state.
    ///
    /// If the refrsh process fails due to core-related issues (store errors,
    /// internal logic, ...) the process is aborted and the error propagated.
    ///
    /// If the refresh process encounters an agent error (invalid response or state,
    /// network issue, ...) the error is NOT propagated and is instead accounted for
    /// as part of the state refersh operation.
    pub fn fetch(
        &self,
        cluster: ClusterDiscovery,
        lock: NonBlockingLockWatcher,
        span: &mut Span,
    ) -> Result<()> {
        span.log(Log::new().log("stage", "fetch"));
        let _timer = FETCHER_DURATION.start_timer();
        self.fetch_checked(cluster, lock, span).map_err(|error| {
            FETCHER_ERRORS_COUNT.inc();
            error
        })
    }

    /// Wrapped version of `fetch` so stats can be accounted for once.
    fn fetch_checked(
        &self,
        cluster: ClusterDiscovery,
        lock: NonBlockingLockWatcher,
        span: &mut Span,
    ) -> Result<()> {
        let cluster_id = cluster.cluster_id;
        debug!(self.logger, "Refreshing cluster state"; "cluster_id" => &cluster_id);
        let mut id_checker = ClusterIdentityChecker::new(cluster_id.clone(), cluster.display_name);
        self.store
            .cluster(cluster_id.clone())
            .mark_stale(span.context().clone())
            .with_context(|_| ErrorKind::StoreWrite("cluster staleness"))?;

        for node in cluster.nodes {
            // Exit early if lock was lost.
            if !lock.inspect() {
                span.log(Log::new().log("abbandoned", "lock lost"));
                warn!(
                    self.logger,
                    "Cluster fetcher lock lost, skipping futher nodes";
                    "cluster_id" => &cluster_id,
                );
                return Ok(());
            }
            self.process_target(&cluster_id, &node, &mut id_checker, span)?;
        }
        Ok(())
    }

    fn process_target(
        &self,
        cluster: &str,
        node: &str,
        id_checker: &mut ClusterIdentityChecker,
        span: &mut Span,
    ) -> Result<()> {
        let client = HttpClient::make(
            node.to_string(),
            self.timeout,
            self.logger.clone(),
            Arc::clone(&self.tracer),
        )
        .with_context(|_| ErrorKind::AgentConnect(node.to_string()))?;
        let mut agent = Agent::new(cluster.to_string(), node.to_string(), AgentStatus::Up);
        let result =
            self.agent
                .process_agent_info(&client, cluster.to_string(), node.to_string(), span);
        if let Err(error) = result {
            let message = format_fail(&error);
            agent.status = AgentStatus::AgentDown(message);
            self.agent.process_agent(agent, span)?;
            if error.kind().is_agent() {
                return Ok(());
            }
            return Err(error);
        };

        let result = self.node.process_node(&client, id_checker, span);
        if let Err(error) = result {
            let message = format_fail(&error);
            agent.status = AgentStatus::NodeDown(message);
            self.agent.process_agent(agent, span)?;
            if error.kind().is_agent() {
                return Ok(());
            }
            return Err(error);
        };

        let result = self.shard.process_shards(&client, cluster, node, span);
        if let Err(error) = result {
            let message = format_fail(&error);
            agent.status = AgentStatus::NodeDown(message);
            self.agent.process_agent(agent, span)?;
            if error.kind().is_agent() {
                return Ok(());
            }
            return Err(error);
        };

        self.agent.process_agent(agent, span)
    }
}
