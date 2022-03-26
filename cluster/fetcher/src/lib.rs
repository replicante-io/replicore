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
use replicante_models_core::agent::Agent;
use replicante_models_core::agent::AgentStatus;
use replicante_models_core::scope::Namespace;
use replicante_service_coordinator::NonBlockingLockWatcher;
use replicante_store_primary::store::Store as PrimaryStore;
use replicante_stream_events::Stream as EventsStream;
use replicante_util_failure::failure_info;

use replicore_cluster_view::ClusterView;
use replicore_cluster_view::ClusterViewBuilder;

mod actions;
mod agent;
mod error;
mod metrics;
mod node;
mod shard;

use self::actions::ActionsFetcher;
use self::agent::AgentFetcher;
use self::metrics::FETCHER_DURATION;
use self::metrics::FETCHER_ERRORS_COUNT;
use self::node::NodeFetcher;
use self::shard::ShardFetcher;

pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;
pub use self::metrics::register_metrics;

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

/// Agent state and actions fetching and processing logic.
///
/// Fetches agent data to "refresh" the persisted view of cluster nodes.
/// See bin/replicante/tasks/cluster_refresh/mod.rs for details on the sync process.
pub struct Fetcher {
    actions: ActionsFetcher,
    agent: AgentFetcher,
    logger: Logger,
    node: NodeFetcher,
    primary_store: PrimaryStore,
    shard: ShardFetcher,
    timeout: Duration,
    tracer: Arc<Tracer>,
}

impl Fetcher {
    pub fn new(
        logger: Logger,
        events: EventsStream,
        primary_store: PrimaryStore,
        timeout: Duration,
        tracer: Arc<Tracer>,
    ) -> Fetcher {
        let actions = ActionsFetcher::new(events.clone(), primary_store.clone(), logger.clone());
        let agent = AgentFetcher::new(events.clone(), primary_store.clone());
        let node = NodeFetcher::new(events.clone(), primary_store.clone());
        let shard = ShardFetcher::new(events, primary_store.clone());
        Fetcher {
            actions,
            agent,
            logger,
            node,
            primary_store,
            shard,
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
    pub fn fetch(
        &self,
        ns: Namespace,
        cluster_view: &ClusterView,
        new_cluster_view: &mut ClusterViewBuilder,
        refresh_id: i64,
        lock: NonBlockingLockWatcher,
        span: &mut Span,
    ) -> Result<()> {
        span.log(Log::new().log("stage", "fetch"));
        let _timer = FETCHER_DURATION.start_timer();
        self.fetch_inner(ns, cluster_view, new_cluster_view, refresh_id, lock, span)
            .map_err(|error| {
                FETCHER_ERRORS_COUNT.inc();
                error
            })
    }

    /// Wrapped version of `fetch` so stats can be accounted for once.
    fn fetch_inner(
        &self,
        ns: Namespace,
        cluster_view: &ClusterView,
        new_cluster_view: &mut ClusterViewBuilder,
        refresh_id: i64,
        lock: NonBlockingLockWatcher,
        span: &mut Span,
    ) -> Result<()> {
        let cluster_id = &cluster_view.cluster_id;
        debug!(self.logger, "Refreshing cluster state"; "cluster_id" => cluster_id);
        let mut id_checker = ClusterIdentityChecker::new(
            cluster_id.clone(),
            cluster_view.discovery.display_name.clone(),
        );
        self.primary_store
            .cluster(ns.ns_id.clone(), cluster_id.clone())
            .mark_stale(span.context().clone())
            .with_context(|_| ErrorKind::PrimaryStoreWrite("cluster staleness"))?;

        for agent_id in &cluster_view.discovery.nodes {
            // Exit early if lock was lost.
            if !lock.inspect() {
                span.log(Log::new().log("abbandoned", "lock lost"));
                warn!(
                    self.logger,
                    "Cluster fetcher lock lost, skipping futher nodes";
                    "namespace" => &ns.ns_id,
                    "cluster_id" => cluster_id,
                    "agent_id" => agent_id,
                );
                return Ok(());
            }

            // Process the target node and inspect the result.
            // If an error within Replicante Core is reported pass it back to the caller
            // and abort the refresh operation, otherwise update the agent status.
            let target = self.process_target(
                &ns,
                cluster_view,
                new_cluster_view,
                agent_id,
                refresh_id,
                &mut id_checker,
                span,
            );
            let agent_status = match target {
                Err(error) => match error.agent_status() {
                    None => return Err(error),
                    Some(status) => {
                        // Log error at debug level and send to sentry as debug.
                        let mut event = sentry::integrations::failure::event_from_fail(&error);
                        event.level = sentry::Level::Debug;
                        sentry::capture_event(event);
                        debug!(
                            self.logger,
                            "Cluster sync operation not successful";
                            "namespace" => &ns.ns_id,
                            "cluster_id" => cluster_id,
                            "agent_id" => &agent_id,
                            failure_info(&error),
                        );
                        status
                    }
                },
                Ok(()) => AgentStatus::Up,
            };
            self.agent.process_agent(
                cluster_view,
                new_cluster_view,
                Agent::new(cluster_id.to_string(), agent_id.to_string(), agent_status),
                span,
            )?;
        }
        Ok(())
    }

    fn process_target(
        &self,
        ns: &Namespace,
        cluster_view: &ClusterView,
        new_cluster_view: &mut ClusterViewBuilder,
        node: &str,
        refresh_id: i64,
        id_checker: &mut ClusterIdentityChecker,
        span: &mut Span,
    ) -> Result<()> {
        let client = HttpClient::new(
            ns,
            node.to_string(),
            self.timeout,
            self.logger.clone(),
            Arc::clone(&self.tracer),
        )
        .with_context(|_| ErrorKind::AgentConnect(node.to_string()))?;

        self.agent
            .process_agent_info(&client, cluster_view, new_cluster_view, node.to_string(), span)?;
        self.node
            .process_node(&client, cluster_view, new_cluster_view, id_checker, span)?;
        self.shard
            .process_shards(&client, cluster_view, new_cluster_view, node, span)?;
        self.actions
            .sync(&client, &cluster_view.cluster_id, node, refresh_id, span)?;

        Ok(())
    }
}
