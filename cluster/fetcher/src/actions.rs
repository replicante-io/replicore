use std::collections::HashSet;

use failure::Fail;
use failure::ResultExt;
use opentracingrust::Span;
use slog::debug;
use slog::info;
use slog::Logger;
use uuid::Uuid;

use replicante_agent_client::Client;
use replicante_models_agent::actions::api::ActionScheduleRequest;
use replicante_models_core::actions::node::Action;
use replicante_models_core::actions::node::ActionState;
use replicante_models_core::actions::node::ActionSyncSummary;
use replicante_models_core::cluster::OrchestrateReportBuilder;
use replicante_models_core::events::Event;
use replicante_store_primary::store::Store;
use replicante_stream_events::EmitMessage;
use replicante_stream_events::Stream as EventsStream;
use replicante_util_failure::SerializableFail;
use replicore_cluster_view::ClusterView;
use replicore_cluster_view::ClusterViewBuilder;

use crate::metrics::FETCHER_ACTIONS_SYNCED;
use crate::metrics::FETCHER_ACTION_SCHEDULE_DUPLICATE;
use crate::metrics::FETCHER_ACTION_SCHEDULE_ERROR;
use crate::metrics::FETCHER_ACTION_SCHEDULE_TOTAL;
use crate::ErrorKind;
use crate::Result;

const MAX_SCHEDULE_ATTEMPTS: i32 = 10;

/// Actions fetch and sync processing.
pub(crate) struct ActionsFetcher {
    events: EventsStream,
    logger: Logger,
    store: Store,
}

impl ActionsFetcher {
    pub fn new(events: EventsStream, store: Store, logger: Logger) -> ActionsFetcher {
        ActionsFetcher {
            events,
            logger,
            store,
        }
    }

    /// Sync the actions for the given node.
    ///
    /// Actions sync operates in the following order:
    ///
    /// 1. Fetch all actions from the agent.
    /// 2. For each action from the agent:
    ///    - If ID in cluster view => get action record from the DB and update it.
    ///    - If ID not in cluster view => add new action record to the DB.
    /// 3. For each action in the cluster view but not returned by agent:
    ///    - If action is pending schedule => schedule action with the agent.
    ///    - If action is running => update the action as lost.
    pub fn sync(
        &self,
        client: &dyn Client,
        cluster_view: &ClusterView,
        new_cluster_view: &mut ClusterViewBuilder,
        report: &mut OrchestrateReportBuilder,
        agent_id: &str,
        span: &mut Span,
    ) -> Result<()> {
        // Step 1: fetch agent info.
        let remote_ids = self.remote_ids(client, &cluster_view.cluster_id, agent_id, span)?;

        // Step 2: sync agent with core.
        let mut synced_actions = 0.0;
        for action_id in &remote_ids {
            self.sync_agent_action(
                client,
                cluster_view,
                new_cluster_view,
                agent_id,
                *action_id,
                span,
            )
            .map_err(|error| {
                FETCHER_ACTIONS_SYNCED.observe(synced_actions);
                error
            })?;
            synced_actions += 1.0;
        }
        FETCHER_ACTIONS_SYNCED.observe(synced_actions);

        // Step 3: sync core with agent.
        let actions = match cluster_view.unfinished_actions_on_node(agent_id) {
            Some(actions) => actions,
            None => {
                debug!(
                    self.logger,
                    "No unfinished actions for node to sync";
                    "namespace" => &cluster_view.namespace,
                    "cluster_id" => &cluster_view.cluster_id,
                    "node_id" => agent_id,
                );
                return Ok(());
            }
        };
        for action_summary in actions {
            // Skip actions processed while looking at agent actions.
            if remote_ids.contains(&action_summary.action_id) {
                continue;
            }
            self.sync_core_action(
                client,
                cluster_view,
                new_cluster_view,
                report,
                agent_id,
                action_summary,
                span,
            )?;
        }
        Ok(())
    }

    /// Update an action in the primary store so we can stop thinking of it.
    fn action_lost(
        &self,
        cluster_view: &ClusterView,
        report: &mut OrchestrateReportBuilder,
        node_id: &str,
        action_summary: &ActionSyncSummary,
        span: &mut Span,
    ) -> Result<()> {
        // Get the action to emit the event correctly.
        let span_context = span.context();
        let action = self
            .store
            .action(cluster_view.cluster_id.clone(), action_summary.action_id)
            .get(span_context.clone())
            .with_context(|_| ErrorKind::PrimaryStoreRead("action"))?;
        let mut action = match action {
            // Can't lose an action we don't know about.
            None => return Ok(()),
            Some(action) => action,
        };
        action.finish(ActionState::Lost);
        report.node_action_lost();

        // Emit action lost event.
        let event = Event::builder().action().lost(action.clone());
        let code = event.code();
        let stream_key = event.entity_id().partition_key();
        let event = EmitMessage::with(stream_key, event)
            .with_context(|_| ErrorKind::EventEmit(code))?
            .trace(span_context.clone());
        self.events
            .emit(event)
            .with_context(|_| ErrorKind::EventEmit(code))?;

        // Persist lost action.
        self.store
            .persist()
            .action(action, span_context.clone())
            .with_context(|_| ErrorKind::PrimaryStoreWrite("action"))?;
        debug!(
            self.logger,
            "Found lost action";
            "namespace" => &cluster_view.namespace,
            "cluster_id" => &cluster_view.cluster_id,
            "node_id" => node_id,
            "action_id" => action_summary.action_id.to_string(),
        );
        Ok(())
    }

    /// Schedule a pending action with an agent.
    ///
    /// On success the state of the action is not updated so the next sync can do it.
    /// This allows the system to auto-retry schedule attempts that appear successful but are not.
    fn action_schedule(
        &self,
        client: &dyn Client,
        cluster_view: &ClusterView,
        report: &mut OrchestrateReportBuilder,
        node_id: &str,
        action_summary: &ActionSyncSummary,
        span: &mut Span,
    ) -> Result<()> {
        // Get the action to schedule.
        let span_context = span.context();
        let action = self
            .store
            .action(cluster_view.cluster_id.clone(), action_summary.action_id)
            .get(span_context.clone())
            .with_context(|_| ErrorKind::PrimaryStoreRead("action"))?
            .ok_or_else(|| {
                ErrorKind::ExpectedActionNotFound(
                    cluster_view.namespace.clone(),
                    cluster_view.cluster_id.clone(),
                    action_summary.action_id,
                )
            })?;

        // Schedule the action with the agent.
        let request = ActionScheduleRequest {
            action_id: Some(action.action_id),
            args: action.args.clone(),
            created_ts: Some(action.created_ts),
            requester: Some(action.requester.clone()),
        };
        let result = client.schedule_action(
            &action.kind,
            &action.headers,
            request,
            Some(span.context().clone()),
        );
        report.node_action_scheduled();
        FETCHER_ACTION_SCHEDULE_TOTAL.inc();

        // Handle scheduling error to re-try later.
        match result {
            Err(error) if !error.kind().is_duplicate_action() => {
                // After a while give up on trying to schedule actions.
                let mut action = action;
                let payload = SerializableFail::from(&error);
                let payload = serde_json::to_value(payload).expect("errors must always serialise");
                action.schedule_attempt += 1;
                action.state_payload = Some(payload);
                // TODO: make MAX_SCHEDULE_ATTEMPTS a namespace configuration once namespaces exist.
                if action.schedule_attempt > MAX_SCHEDULE_ATTEMPTS {
                    action.finish(ActionState::Failed);
                }
                self.store
                    .persist()
                    .action(action, span.context().clone())
                    .with_context(|_| ErrorKind::PrimaryStoreWrite("action"))?;
                FETCHER_ACTION_SCHEDULE_ERROR.inc();
                report.node_action_schedule_failed();
                let error =
                    error.context(ErrorKind::AgentDown("action request", node_id.to_string()));
                return Err(error.into());
            }
            Err(error) if error.kind().is_duplicate_action() => {
                info!(
                    self.logger,
                    "Ignored duplicate action scheduling attempt";
                    "namespace" => &cluster_view.namespace,
                    "cluster_id" => &cluster_view.cluster_id,
                    "node_id" => node_id,
                    "action_id" => action.action_id.to_string(),
                );
                FETCHER_ACTION_SCHEDULE_DUPLICATE.inc();
            }
            _ => (),
        };

        // On scheduling success reset attempt counter, if needed.
        if action.schedule_attempt != 0 {
            let mut action = action;
            action.schedule_attempt = 0;
            action.state_payload = None;
            self.store
                .persist()
                .action(action, span_context.clone())
                .with_context(|_| ErrorKind::PrimaryStoreWrite("action"))?;
        }
        Ok(())
    }

    /// Fetch all action IDs currently available on the agent.
    fn remote_ids(
        &self,
        client: &dyn Client,
        cluster_id: &str,
        node_id: &str,
        span: &mut Span,
    ) -> Result<Vec<Uuid>> {
        debug!(
            self.logger,
            "Retrieving action IDs from agent";
            "cluster_id" => cluster_id,
            "node_id" => node_id,
        );
        let mut duplicate_ids = HashSet::new();
        let mut remote_ids = Vec::new();
        let queue = client
            .actions_queue(Some(span.context().clone()))
            .with_context(|_| ErrorKind::AgentDown("actions queue", node_id.to_string()))?;
        let finished = client
            .actions_finished(Some(span.context().clone()))
            .with_context(|_| ErrorKind::AgentDown("actions finished", node_id.to_string()))?;
        for list in &[queue, finished] {
            for action in list {
                if !duplicate_ids.contains(&action.id) {
                    duplicate_ids.insert(action.id);
                    remote_ids.push(action.id);
                }
            }
        }
        Ok(remote_ids)
    }

    /// Sync a single action from an agent to core.
    fn sync_agent_action(
        &self,
        client: &dyn Client,
        cluster_view: &ClusterView,
        new_cluster_view: &mut ClusterViewBuilder,
        node_id: &str,
        action_id: Uuid,
        span: &mut Span,
    ) -> Result<()> {
        // Fetch and process the action.
        let span_context = span.context().clone();
        let info = client.action_info(&action_id, Some(span_context.clone()));
        let info = match info {
            Err(ref error) if error.not_found() => return Ok(()),
            _ => info.with_context(|_| ErrorKind::AgentDown("action info", node_id.to_string())),
        }?;
        let action_agent = Action::new(
            cluster_view.cluster_id.clone(),
            node_id.to_string(),
            info.action,
        );
        let action_db = self
            .store
            .action(cluster_view.cluster_id.clone(), action_id)
            .get(span_context)
            .with_context(|_| ErrorKind::PrimaryStoreRead("action"))?;

        // Emit action-related events.
        let emit_action_finished;
        // --> New or change events.
        match action_db {
            None => {
                emit_action_finished = true;
                let event = Event::builder().action().new_action(action_agent.clone());
                let code = event.code();
                let stream_key = event.entity_id().partition_key();
                let event = EmitMessage::with(stream_key, event)
                    .with_context(|_| ErrorKind::EventEmit(code))?
                    .trace(span.context().clone());
                self.events
                    .emit(event)
                    .with_context(|_| ErrorKind::EventEmit(code))?;
            }
            Some(action_db) => {
                // Skip updates for already finished and unchanged actions.
                if action_db.finished_ts.is_some() || action_agent == action_db {
                    debug!(
                        self.logger,
                        "Skipping sync for finished action";
                        "namespace" => &cluster_view.namespace,
                        "cluster_id" => &cluster_view.cluster_id,
                        "node_id" => node_id,
                        "action_id" => action_agent.action_id.to_string(),
                    );
                    return Ok(());
                }

                emit_action_finished = action_agent.finished_ts.is_some();
                let event = Event::builder()
                    .action()
                    .changed(action_db, action_agent.clone());
                let code = event.code();
                let stream_key = event.entity_id().partition_key();
                let event = EmitMessage::with(stream_key, event)
                    .with_context(|_| ErrorKind::EventEmit(code))?
                    .trace(span.context().clone());
                self.events
                    .emit(event)
                    .with_context(|_| ErrorKind::EventEmit(code))?;
            }
        };

        // --> Action finished event (if action was not already finished).
        if emit_action_finished && action_agent.finished_ts.is_some() {
            let event = Event::builder().action().finished(action_agent.clone());
            let code = event.code();
            let stream_key = event.entity_id().partition_key();
            let event = EmitMessage::with(stream_key, event)
                .with_context(|_| ErrorKind::EventEmit(code))?
                .trace(span.context().clone());
            self.events
                .emit(event)
                .with_context(|_| ErrorKind::EventEmit(code))?;
        }

        // --> Action history event.
        let event = Event::builder().action().history(
            action_agent.cluster_id.clone(),
            action_agent.node_id.clone(),
            action_agent.action_id,
            action_agent.finished_ts,
            info.history,
        );
        let code = event.code();
        let stream_key = event.entity_id().partition_key();
        let event = EmitMessage::with(stream_key, event)
            .with_context(|_| ErrorKind::EventEmit(code))?
            .trace(span.context().clone());
        self.events
            .emit(event)
            .with_context(|_| ErrorKind::EventEmit(code))?;

        // Update the new cluster view.
        if action_agent.finished_ts.is_none() {
            new_cluster_view
                .action(ActionSyncSummary::from(&action_agent))
                .map_err(crate::error::AnyWrap::from)
                .context(ErrorKind::ClusterViewUpdate)?;
        }

        // Persist the new action information.
        self.store
            .persist()
            .action(action_agent, span.context().clone())
            .with_context(|_| ErrorKind::PrimaryStoreWrite("action"))?;
        Ok(())
    }

    /// Sync a single action from core to an agent.
    #[allow(clippy::too_many_arguments)]
    fn sync_core_action(
        &self,
        client: &dyn Client,
        cluster_view: &ClusterView,
        new_cluster_view: &mut ClusterViewBuilder,
        report: &mut OrchestrateReportBuilder,
        node_id: &str,
        action_summary: &ActionSyncSummary,
        span: &mut Span,
    ) -> Result<()> {
        // Append pending actions to the new cluster view as well.
        let pending_action = matches!(
            action_summary.state,
            ActionState::PendingApprove | ActionState::PendingSchedule
        );
        if pending_action {
            new_cluster_view
                .action(action_summary.clone())
                .map_err(crate::error::AnyWrap::from)
                .context(ErrorKind::ClusterViewUpdate)?;
        }

        // Process core actions based on their state.
        match action_summary.state {
            // Schedule pending actions with the agent.
            ActionState::PendingSchedule => {
                self.action_schedule(client, cluster_view, report, node_id, action_summary, span)
            }
            // Running actions no longer known to the agent are lost.
            state if state.is_running() => {
                self.action_lost(cluster_view, report, node_id, action_summary, span)
            }
            // Skip all other actions.
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::Utc;
    use opentracingrust::tracers::NoopTracer;
    use serde_json::json;
    use slog::o;
    use slog::Discard;
    use slog::Logger;
    use uuid::Uuid;

    use replicante_agent_client::mock::MockClient;
    use replicante_models_agent::actions::api::ActionInfoResponse;
    use replicante_models_agent::actions::ActionListItem;
    use replicante_models_agent::actions::ActionModel as AgentActionModel;
    use replicante_models_agent::actions::ActionState as ActionStateAgent;
    use replicante_models_core::actions::node::Action as CoreAction;
    use replicante_models_core::actions::node::ActionRequester;
    use replicante_models_core::actions::node::ActionState as ActionStateCore;
    use replicante_models_core::actions::node::ActionSyncSummary;
    use replicante_models_core::cluster::discovery::ClusterDiscovery;
    use replicante_models_core::cluster::ClusterSettings;
    use replicante_models_core::cluster::OrchestrateReportBuilder;
    use replicante_store_primary::mock::Mock as StoreMock;
    use replicante_stream_events::Stream as EventsStream;
    use replicore_cluster_view::ClusterView;
    use replicore_cluster_view::ClusterViewBuilder;

    use super::ActionsFetcher;

    lazy_static::lazy_static! {
        static ref UUID1: Uuid = "a7514ce6-48f4-4f9d-bb22-78cbfc37c664".parse().unwrap();
        static ref UUID2: Uuid = "9084aec4-2234-4b9b-8a5d-aac914127255".parse().unwrap();
        static ref UUID3: Uuid = "be6ddf09-5c16-4be4-84dd-d03586eb1fc3".parse().unwrap();
        static ref UUID4: Uuid = "390ef9ab-ce0e-468e-977d-65873274c448".parse().unwrap();
        static ref UUID5: Uuid = "e5a023c6-78a3-4eb0-bc8f-6c5d057964ef".parse().unwrap();
        static ref UUID6: Uuid = "b9754ca6-824f-4796-8982-583888d2de19".parse().unwrap();
    }

    fn cluster_view_builder() -> ClusterViewBuilder {
        let discovery = ClusterDiscovery::new("test", vec![]);
        let settings = ClusterSettings::synthetic("test", "test");
        ClusterView::builder(settings, discovery).expect("mock cluster view to build")
    }

    fn mock_agent_action(id: Uuid, finished: bool) -> AgentActionModel {
        let created_ts = Utc::now();
        let finished_ts = if finished { Some(Utc::now()) } else { None };
        let scheduled_ts = Utc::now();
        AgentActionModel {
            args: json!({}),
            created_ts,
            finished_ts,
            headers: HashMap::new(),
            id,
            kind: "action".into(),
            requester: ActionRequester::AgentApi,
            scheduled_ts,
            state: if finished {
                ActionStateAgent::Done
            } else {
                ActionStateAgent::New
            },
            state_payload: None,
        }
    }

    fn mock_cluster_view() -> ClusterView {
        mock_cluster_view_with_actions(vec![])
    }

    fn mock_cluster_view_with_actions(actions: Vec<ActionSyncSummary>) -> ClusterView {
        let discovery = ClusterDiscovery::new("test", vec![]);
        let settings = ClusterSettings::synthetic("test", "test");
        let mut view =
            ClusterView::builder(settings, discovery).expect("mock cluster view to build");

        for action in actions {
            view.action(action).expect("action to be added to view");
        }
        view.build()
    }

    #[test]
    fn fetch_remote_ids() {
        let mut client = MockClient::new(
            || panic!("unused in these tests"),
            || panic!("unused in these tests"),
            || panic!("unused in these tests"),
        );
        client.actions_queue = vec![ActionListItem {
            id: *UUID1,
            kind: "test".into(),
            state: ActionStateAgent::New,
        }];
        client.actions_finished = vec![
            ActionListItem {
                id: *UUID2,
                kind: "test".into(),
                state: ActionStateAgent::Done,
            },
            ActionListItem {
                id: *UUID3,
                kind: "test".into(),
                state: ActionStateAgent::Done,
            },
        ];
        let store = StoreMock::default().store();
        let stream = EventsStream::mock();
        let fetcher = ActionsFetcher::new(stream, store, Logger::root(Discard, o!()));
        let (tracer, _) = NoopTracer::new();
        let mut span = tracer.span("test");
        let ids = fetcher
            .remote_ids(&client, "test", "node", &mut span)
            .expect("failed to fetch ids");
        assert_eq!(ids, vec![*UUID1, *UUID2, *UUID3]);
    }

    // This test cover the case of actions being finished between
    // the call to /finish and the call to /queue.
    #[test]
    fn fetch_remote_ids_with_duplicate_actions() {
        let mut client = MockClient::new(
            || panic!("unused in these tests"),
            || panic!("unused in these tests"),
            || panic!("unused in these tests"),
        );
        client.actions_queue = vec![
            ActionListItem {
                id: *UUID3,
                kind: "test".into(),
                state: ActionStateAgent::Running,
            },
            ActionListItem {
                id: *UUID2,
                kind: "test".into(),
                state: ActionStateAgent::New,
            },
            ActionListItem {
                id: *UUID1,
                kind: "test".into(),
                state: ActionStateAgent::New,
            },
        ];
        client.actions_finished = vec![
            ActionListItem {
                id: *UUID2,
                kind: "test".into(),
                state: ActionStateAgent::Done,
            },
            ActionListItem {
                id: *UUID3,
                kind: "test".into(),
                state: ActionStateAgent::Done,
            },
            ActionListItem {
                id: *UUID4,
                kind: "test".into(),
                state: ActionStateAgent::Done,
            },
        ];
        let store = StoreMock::default().store();
        let stream = EventsStream::mock();
        let fetcher = ActionsFetcher::new(stream, store, Logger::root(Discard, o!()));
        let (tracer, _) = NoopTracer::new();
        let mut span = tracer.span("test");
        let ids = fetcher
            .remote_ids(&client, "test", "node", &mut span)
            .expect("failed to fetch ids");
        assert_eq!(ids, vec![*UUID3, *UUID2, *UUID1, *UUID4]);
    }

    #[test]
    fn sync_action_new() {
        // Set up client.
        let mut client = MockClient::new(
            || panic!("unused in these tests"),
            || panic!("unused in these tests"),
            || panic!("unused in these tests"),
        );
        let mut action = mock_agent_action(*UUID1, false);
        action.state = ActionStateAgent::New;
        let info = ActionInfoResponse {
            action: action.clone(),
            history: Vec::new(),
        };
        client.actions.insert(*UUID1, info);

        // Set up fetcher and sync an action.
        let cluster_view = mock_cluster_view();
        let mut new_cluster_view = cluster_view_builder();
        let store = StoreMock::default();
        let stream = EventsStream::mock();
        let fetcher =
            ActionsFetcher::new(stream, store.clone().store(), Logger::root(Discard, o!()));
        let (tracer, _) = NoopTracer::new();
        let mut span = tracer.span("test");
        fetcher
            .sync_agent_action(
                &client,
                &cluster_view,
                &mut new_cluster_view,
                "node",
                *UUID1,
                &mut span,
            )
            .expect("action sync failed");

        // Assert sync result.
        let action = {
            let store = store.state.lock().expect("MockStore state lock poisoned");
            store
                .actions
                .get(&("test".into(), "node".into(), *UUID1))
                .expect("expected action not found")
                .clone()
        };
        assert_eq!(action.action_id, *UUID1);
        assert_eq!(action.state, ActionStateCore::New);
    }

    #[test]
    fn sync_action_update() {
        // Fill the store with an action.
        let mut action = mock_agent_action(*UUID1, false);
        action.state = ActionStateAgent::New;
        let store = StoreMock::default();
        store
            .store()
            .persist()
            .action(CoreAction::new("test", "node", action.clone()), None)
            .expect("failed to add action to store");

        // Set up client.
        let mut client = MockClient::new(
            || panic!("unused in these tests"),
            || panic!("unused in these tests"),
            || panic!("unused in these tests"),
        );
        action.state = ActionStateAgent::Running;
        let info = ActionInfoResponse {
            action: action.clone(),
            history: Vec::new(),
        };
        client.actions.insert(*UUID1, info);

        // Set up fetcher and sync an action.
        let cluster_view = mock_cluster_view();
        let mut new_cluster_view = cluster_view_builder();
        let stream = EventsStream::mock();
        let fetcher =
            ActionsFetcher::new(stream, store.clone().store(), Logger::root(Discard, o!()));
        let (tracer, _) = NoopTracer::new();
        let mut span = tracer.span("test");
        fetcher
            .sync_agent_action(
                &client,
                &cluster_view,
                &mut new_cluster_view,
                "node",
                *UUID1,
                &mut span,
            )
            .expect("action sync failed");

        // Assert sync result.
        let action = {
            let store = store.state.lock().expect("MockStore state lock poisoned");
            store
                .actions
                .get(&("test".into(), "node".into(), *UUID1))
                .expect("expected action not found")
                .clone()
        };
        assert_eq!(action.action_id, *UUID1);
        assert_eq!(action.state, ActionStateCore::Running);
    }

    #[test]
    fn sync_lost_actions() {
        // Fill the store with finished and unfinished actions.
        let store = StoreMock::default();
        let action = mock_agent_action(*UUID1, true);
        store
            .store()
            .persist()
            .action(CoreAction::new("test", "node", action), None)
            .expect("failed to add action to store");
        let action = mock_agent_action(*UUID2, true);
        store
            .store()
            .persist()
            .action(CoreAction::new("test", "node", action), None)
            .expect("failed to add action to store");
        let action = mock_agent_action(*UUID3, false);
        store
            .store()
            .persist()
            .action(CoreAction::new("test", "node", action), None)
            .expect("failed to add action to store");
        let action = mock_agent_action(*UUID4, false);
        store
            .store()
            .persist()
            .action(CoreAction::new("test", "node", action), None)
            .expect("failed to add action to store");

        // Set up client.
        let client = MockClient::new(
            || panic!("unused in these tests"),
            || panic!("unused in these tests"),
            || panic!("unused in these tests"),
        );

        // Set up fetcher and sync an action.
        let cluster_view = mock_cluster_view_with_actions(vec![
            ActionSyncSummary {
                cluster_id: "test".into(),
                node_id: "node".into(),
                action_id: *UUID1,
                state: ActionStateCore::Done,
            },
            ActionSyncSummary {
                cluster_id: "test".into(),
                node_id: "node".into(),
                action_id: *UUID2,
                state: ActionStateCore::Done,
            },
            ActionSyncSummary {
                cluster_id: "test".into(),
                node_id: "node".into(),
                action_id: *UUID3,
                state: ActionStateCore::New,
            },
            ActionSyncSummary {
                cluster_id: "test".into(),
                node_id: "node".into(),
                action_id: *UUID4,
                state: ActionStateCore::New,
            },
        ]);
        let mut new_cluster_view = cluster_view_builder();
        let stream = EventsStream::mock();
        let fetcher =
            ActionsFetcher::new(stream, store.clone().store(), Logger::root(Discard, o!()));
        let (tracer, _) = NoopTracer::new();
        let mut span = tracer.span("test");
        fetcher
            .sync(
                &client,
                &cluster_view,
                &mut new_cluster_view,
                &mut OrchestrateReportBuilder::new(),
                "node",
                &mut span,
            )
            .expect("actions sync failed");

        let action = store
            .store()
            .action("test".to_string(), *UUID1)
            .get(None)
            .expect("action to be in store")
            .expect("action to be in store");
        assert_eq!(action.state, ActionStateCore::Done);
        let action = store
            .store()
            .action("test".to_string(), *UUID2)
            .get(None)
            .expect("action to be in store")
            .expect("action to be in store");
        assert_eq!(action.state, ActionStateCore::Done);
        let action = store
            .store()
            .action("test".to_string(), *UUID3)
            .get(None)
            .expect("action to be in store")
            .expect("action to be in store");
        assert_eq!(action.state, ActionStateCore::Lost);
        let action = store
            .store()
            .action("test".to_string(), *UUID4)
            .get(None)
            .expect("action to be in store")
            .expect("action to be in store");
        assert_eq!(action.state, ActionStateCore::Lost);
    }

    #[test]
    fn sync_schedule_pending_actions() {
        // Fill the store with pending approve and pending schedule actions.
        let store = StoreMock::default();
        let action = mock_agent_action(*UUID1, false);
        let mut action = CoreAction::new("test", "node", action);
        action.state = ActionStateCore::PendingApprove;
        store
            .store()
            .persist()
            .action(action, None)
            .expect("failed to add action to store");

        let action = mock_agent_action(*UUID2, false);
        let mut action = CoreAction::new("test", "node", action);
        action.state = ActionStateCore::PendingSchedule;
        store
            .store()
            .persist()
            .action(action, None)
            .expect("failed to add action to store");

        // Set up client.
        let client = MockClient::new(
            || panic!("unused in these tests"),
            || panic!("unused in these tests"),
            || panic!("unused in these tests"),
        );

        // Set up fetcher and sync an action.
        let cluster_view = mock_cluster_view_with_actions(vec![
            ActionSyncSummary {
                cluster_id: "test".into(),
                node_id: "node".into(),
                action_id: *UUID1,
                state: ActionStateCore::PendingApprove,
            },
            ActionSyncSummary {
                cluster_id: "test".into(),
                node_id: "node".into(),
                action_id: *UUID2,
                state: ActionStateCore::PendingSchedule,
            },
        ]);
        let mut new_cluster_view = cluster_view_builder();
        let stream = EventsStream::mock();
        let fetcher =
            ActionsFetcher::new(stream, store.clone().store(), Logger::root(Discard, o!()));
        let (tracer, _) = NoopTracer::new();
        let mut span = tracer.span("test");
        fetcher
            .sync(
                &client,
                &cluster_view,
                &mut new_cluster_view,
                &mut OrchestrateReportBuilder::new(),
                "node",
                &mut span,
            )
            .expect("actions sync failed");

        // Check request sent to the agent.
        let actions_to_schedule = client
            .actions_to_schedule
            .lock()
            .expect("agent MockClient::actions_to_schedule lock poisoned");
        assert_eq!(actions_to_schedule.len(), 1);
        let (kind, request) = actions_to_schedule
            .get(0)
            .expect("schedule action request")
            .clone();
        assert_eq!(request.action_id, Some(*UUID2));
        assert_eq!(kind, "action");
    }

    #[test]
    fn track_unfinished_actions_in_cluster_view() {
        // Fill DB with finished, running and pending actions.
        let store = StoreMock::default();
        let action = mock_agent_action(*UUID1, false);
        let mut action = CoreAction::new("test", "node", action);
        action.state = ActionStateCore::PendingApprove;
        store
            .store()
            .persist()
            .action(action, None)
            .expect("failed to add action to store");

        let action = mock_agent_action(*UUID2, true);
        let action = CoreAction::new("test", "node", action);
        store
            .store()
            .persist()
            .action(action, None)
            .expect("failed to add action to store");

        let action = mock_agent_action(*UUID3, false);
        let mut action = CoreAction::new("test", "node", action);
        action.state = ActionStateCore::PendingSchedule;
        store
            .store()
            .persist()
            .action(action, None)
            .expect("failed to add action to store");

        let action = mock_agent_action(*UUID4, false);
        let mut action = CoreAction::new("test", "node", action);
        action.state = ActionStateCore::Running;
        store
            .store()
            .persist()
            .action(action, None)
            .expect("failed to add action to store");

        let action = mock_agent_action(*UUID5, false);
        let mut action = CoreAction::new("test", "node", action);
        action.state = ActionStateCore::Running;
        store
            .store()
            .persist()
            .action(action, None)
            .expect("failed to add action to store");

        // Fill the Cluster View with DB actions.
        let cluster_view = mock_cluster_view_with_actions(vec![
            ActionSyncSummary {
                cluster_id: "test".into(),
                node_id: "node".into(),
                action_id: *UUID1,
                state: ActionStateCore::PendingApprove,
            },
            ActionSyncSummary {
                cluster_id: "test".into(),
                node_id: "node".into(),
                action_id: *UUID3,
                state: ActionStateCore::PendingSchedule,
            },
            ActionSyncSummary {
                cluster_id: "test".into(),
                node_id: "node".into(),
                action_id: *UUID4,
                state: ActionStateCore::Running,
            },
            ActionSyncSummary {
                cluster_id: "test".into(),
                node_id: "node".into(),
                action_id: *UUID5,
                state: ActionStateCore::Running,
            },
        ]);

        // Set up client with actions (the running action from DB as finished, new action).
        let mut client = MockClient::new(
            || panic!("unused in these tests"),
            || panic!("unused in these tests"),
            || panic!("unused in these tests"),
        );
        let info = ActionInfoResponse {
            action: mock_agent_action(*UUID2, true),
            history: Vec::new(),
        };
        client.actions.insert(*UUID2, info);
        client.actions_finished.push(ActionListItem {
            id: *UUID2,
            kind: "action".into(),
            state: ActionStateAgent::Done,
        });

        let action = mock_agent_action(*UUID4, true);
        let info = ActionInfoResponse {
            action: action,
            history: Vec::new(),
        };
        client.actions.insert(*UUID4, info);
        client.actions_finished.push(ActionListItem {
            id: *UUID4,
            kind: "action".into(),
            state: ActionStateAgent::Done,
        });

        let mut action = mock_agent_action(*UUID6, false);
        action.state = ActionStateAgent::Running;
        let info = ActionInfoResponse {
            action: action,
            history: Vec::new(),
        };
        client.actions.insert(*UUID6, info);
        client.actions_queue.push(ActionListItem {
            id: *UUID6,
            kind: "action".into(),
            state: ActionStateAgent::Running,
        });

        // Run sync.
        let mut new_cluster_view = cluster_view_builder();
        let stream = EventsStream::mock();
        let fetcher =
            ActionsFetcher::new(stream, store.clone().store(), Logger::root(Discard, o!()));
        let (tracer, _) = NoopTracer::new();
        let mut span = tracer.span("test");
        fetcher
            .sync(
                &client,
                &cluster_view,
                &mut new_cluster_view,
                &mut OrchestrateReportBuilder::new(),
                "node",
                &mut span,
            )
            .expect("actions sync failed");

        // Finish new cluster view and check actions in it.
        let new_cluster_view = new_cluster_view.build();
        assert_eq!(new_cluster_view.actions_unfinished_by_node.len(), 1);
        let actions = new_cluster_view
            .actions_unfinished_by_node
            .get("node")
            .expect("missing actions for expected node");
        assert_eq!(
            *actions,
            vec![
                ActionSyncSummary {
                    cluster_id: "test".into(),
                    node_id: "node".into(),
                    action_id: *UUID6,
                    state: ActionStateCore::Running,
                },
                ActionSyncSummary {
                    cluster_id: "test".into(),
                    node_id: "node".into(),
                    action_id: *UUID1,
                    state: ActionStateCore::PendingApprove,
                },
                ActionSyncSummary {
                    cluster_id: "test".into(),
                    node_id: "node".into(),
                    action_id: *UUID3,
                    state: ActionStateCore::PendingSchedule,
                },
            ]
        );
    }
}
