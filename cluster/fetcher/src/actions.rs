use std::collections::HashSet;

use chrono::Utc;
use failure::Fail;
use failure::ResultExt;
use opentracingrust::Span;
use slog::info;
use slog::Logger;
use uuid::Uuid;

use replicante_agent_client::Client;
use replicante_models_agent::actions::api::ActionScheduleRequest;
use replicante_models_core::actions::Action;
use replicante_models_core::actions::ActionState;
use replicante_models_core::events::Event;
use replicante_store_primary::store::actions::ActionSyncState;
use replicante_store_primary::store::actions::MAX_ACTIONS_STATE_FOR_SYNC;
use replicante_store_primary::store::Store as PrimaryStore;
use replicante_stream_events::EmitMessage;
use replicante_stream_events::Stream as EventsStream;
use replicante_util_failure::SerializableFail;

use crate::metrics::FETCHER_ACTIONS_CHUNKS;
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
    primary_store: PrimaryStore,
}

impl ActionsFetcher {
    pub fn new(
        events: EventsStream,
        primary_store: PrimaryStore,
        logger: Logger,
    ) -> ActionsFetcher {
        ActionsFetcher {
            events,
            logger,
            primary_store,
        }
    }

    /// Sync the actions for the given node.
    pub fn sync(
        &self,
        client: &dyn Client,
        cluster_id: &str,
        agent_id: &str,
        refresh_id: i64,
        span: &mut Span,
    ) -> Result<()> {
        let remote_ids = self.remote_ids(client, agent_id, span)?;
        let sync_ids = self.check_ids_to_sync(cluster_id, agent_id, remote_ids, span)?;
        let sync_size = sync_ids.len();
        for action_info in sync_ids {
            self.sync_action(client, cluster_id, agent_id, action_info, refresh_id, span)?;
        }
        FETCHER_ACTIONS_SYNCED.observe(sync_size as f64);
        self.mark_lost_actions(cluster_id, agent_id, refresh_id, span)?;
        self.schedule_pending(client, cluster_id, agent_id, span)
    }

    /// Check the given remote IDs against the primary store and return a list of IDs to sync.
    fn check_ids_to_sync(
        &self,
        cluster_id: &str,
        node_id: &str,
        remote_ids: Vec<Uuid>,
        span: &mut Span,
    ) -> Result<Vec<(Uuid, ActionSyncState)>> {
        let mut results = Vec::new();
        let chunks = remote_ids.chunks(MAX_ACTIONS_STATE_FOR_SYNC);
        let chunks_count = chunks.len();
        for ids in chunks {
            let mut states = self
                .primary_store
                .actions(cluster_id.to_string())
                .state_for_sync(node_id.to_string(), &ids, span.context().clone())
                .with_context(|_| ErrorKind::PrimaryStoreRead("actions state for sync"))?;
            for id in ids {
                let state = states
                    .remove(id)
                    .expect("state_for_sync did not return all IDs");
                match state {
                    // Once we find the first `Finished` action we stop checking.
                    // This works because of required ordering of actions.
                    // See bin/replicante/src/tasks/cluster_refresh/mod.rs for details on the sync process.
                    ActionSyncState::Finished => return Ok(results),
                    state => results.push((*id, state)),
                }
            }
        }
        FETCHER_ACTIONS_CHUNKS.observe(chunks_count as f64);
        Ok(results)
    }

    /// Mark unfinished actions on the node that were not refreshed as lost.
    fn mark_lost_actions(
        &self,
        cluster_id: &str,
        node_id: &str,
        refresh_id: i64,
        span: &mut Span,
    ) -> Result<()> {
        // Emit acions for each lost action.
        let finished_ts = Utc::now();
        let lost = self
            .primary_store
            .actions(cluster_id.to_string())
            .iter_lost(
                node_id.to_string(),
                refresh_id,
                finished_ts,
                span.context().clone(),
            )
            .with_context(|_| ErrorKind::PrimaryStoreRead("iter lost actions"))?;
        for action in lost {
            let mut action =
                action.with_context(|_| ErrorKind::PrimaryStoreRead("iter lost actions"))?;
            action.finished_ts = Some(finished_ts);
            action.state = ActionState::Lost;
            let event = Event::builder().action().lost(action);
            let code = event.code();
            let stream_key = event.stream_key();
            let event = EmitMessage::with(stream_key, event)
                .with_context(|_| ErrorKind::EventEmit(code))?
                .trace(span.context().clone());
            self.events
                .emit(event)
                .with_context(|_| ErrorKind::EventEmit(code))?;
        }

        // Update these actions to be lost once events have been sent.
        self.primary_store
            .actions(cluster_id.to_string())
            .mark_lost(
                node_id.to_string(),
                refresh_id,
                finished_ts,
                span.context().clone(),
            )
            .with_context(|_| ErrorKind::PrimaryStoreWrite("mark actions as lost"))?;
        // TODO: count lost actions
        Ok(())
    }

    /// Fetch all action IDs currently available on the agent.
    fn remote_ids(&self, client: &dyn Client, node_id: &str, span: &mut Span) -> Result<Vec<Uuid>> {
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

    pub fn schedule_pending(
        &self,
        client: &dyn Client,
        cluster_id: &str,
        agent_id: &str,
        span: &mut Span,
    ) -> Result<()> {
        let actions = self
            .primary_store
            .actions(cluster_id.to_string())
            .pending_schedule(agent_id.to_string(), span.context().clone())
            .with_context(|_| ErrorKind::PrimaryStoreRead("pending actions"))?;
        for action in actions {
            let mut action = action.with_context(|_| ErrorKind::PrimaryStoreRead("action"))?;
            let action_id = action.action_id.to_string();
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
            FETCHER_ACTION_SCHEDULE_TOTAL.inc();
            match result {
                Ok(()) => (),
                Err(error) => match error.kind() {
                    replicante_agent_client::ErrorKind::DuplicateAction => {
                        info!(
                            self.logger,
                            "Ignored duplicate action scheduling attempt";
                            "cluster_id" => cluster_id,
                            "agent_id" => agent_id,
                            "action_id" => action_id,
                        );
                        FETCHER_ACTION_SCHEDULE_DUPLICATE.inc();
                    }
                    _ => {
                        // After a while give up on trying to schedule actions.
                        let payload = SerializableFail::from(&error);
                        let payload =
                            serde_json::to_value(payload).expect("errors must always serialise");
                        action.schedule_attempt += 1;
                        action.state_payload = Some(payload);
                        // TODO: make MAX_SCHEDULE_ATTEMPTS a namespace configuration once namesapces exist.
                        if action.schedule_attempt > MAX_SCHEDULE_ATTEMPTS {
                            action.finish(ActionState::Failed);
                        }
                        self.primary_store
                            .persist()
                            .action(action, span.context().clone())
                            .with_context(|_| ErrorKind::PrimaryStoreWrite("action"))?;
                        FETCHER_ACTION_SCHEDULE_ERROR.inc();
                        let error = error
                            .context(ErrorKind::AgentDown("action request", agent_id.to_string()));
                        return Err(error.into());
                    }
                },
            };
            // Reset schedule attempt count if needed.
            if action.schedule_attempt != 0 {
                action.schedule_attempt = 0;
                action.state_payload = None;
                self.primary_store
                    .persist()
                    .action(action, span.context().clone())
                    .with_context(|_| ErrorKind::PrimaryStoreWrite("action"))?;
            }
        }
        Ok(())
    }

    /// Sync a single action's details.
    fn sync_action(
        &self,
        client: &dyn Client,
        cluster_id: &str,
        node_id: &str,
        action_info: (Uuid, ActionSyncState),
        refresh_id: i64,
        span: &mut Span,
    ) -> Result<()> {
        // Fetch and process the action.
        let (action_id, action_sync_state) = action_info;
        let info = client.action_info(&action_id, Some(span.context().clone()));
        let info = match info {
            Err(ref error) if error.not_found() => return Ok(()),
            _ => info.with_context(|_| ErrorKind::AgentDown("action info", node_id.to_string())),
        }?;
        let action = Action::new(
            cluster_id.to_string(),
            node_id.to_string(),
            refresh_id,
            info.action,
        );

        // Emit action-related events.
        match action_sync_state {
            ActionSyncState::Found(old) => {
                // Using a guard causes issues with ownership so check now for changes
                // compared to our record of the action and conditionally emit an event.
                if action.finished_ts.is_none() && action != old {
                    let event = Event::builder().action().changed(old, action.clone());
                    let code = event.code();
                    let stream_key = event.stream_key();
                    let event = EmitMessage::with(stream_key, event)
                        .with_context(|_| ErrorKind::EventEmit(code))?
                        .trace(span.context().clone());
                    self.events
                        .emit(event)
                        .with_context(|_| ErrorKind::EventEmit(code))?;
                }
            }
            ActionSyncState::NotFound => {
                let event = Event::builder().action().new_action(action.clone());
                let code = event.code();
                let stream_key = event.stream_key();
                let event = EmitMessage::with(stream_key, event)
                    .with_context(|_| ErrorKind::EventEmit(code))?
                    .trace(span.context().clone());
                self.events
                    .emit(event)
                    .with_context(|_| ErrorKind::EventEmit(code))?;
            }
            _ => (),
        };
        if action.finished_ts.is_some() {
            let event = Event::builder().action().finished(action.clone());
            let code = event.code();
            let stream_key = event.stream_key();
            let event = EmitMessage::with(stream_key, event)
                .with_context(|_| ErrorKind::EventEmit(code))?
                .trace(span.context().clone());
            self.events
                .emit(event)
                .with_context(|_| ErrorKind::EventEmit(code))?;
        }

        // Persist the new action information.
        let event = Event::builder().action().history(
            action.cluster_id.clone(),
            action.node_id.clone(),
            action.action_id,
            action.finished_ts,
            info.history,
        );
        let code = event.code();
        let stream_key = event.stream_key();
        let event = EmitMessage::with(stream_key, event)
            .with_context(|_| ErrorKind::EventEmit(code))?
            .trace(span.context().clone());
        self.events
            .emit(event)
            .with_context(|_| ErrorKind::EventEmit(code))?;
        self.primary_store
            .persist()
            .action(action, span.context().clone())
            .with_context(|_| ErrorKind::PrimaryStoreWrite("action"))?;
        Ok(())
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
    use replicante_models_core::actions::Action as CoreAction;
    use replicante_models_core::actions::ActionRequester;
    use replicante_models_core::actions::ActionState as ActionStateCore;
    use replicante_store_primary::mock::Mock as PrimaryStoreMock;
    use replicante_store_primary::store::actions::ActionSyncState;
    use replicante_stream_events::Stream as EventsStream;

    use super::ActionsFetcher;

    lazy_static::lazy_static! {
        static ref UUID1: Uuid = "a7514ce6-48f4-4f9d-bb22-78cbfc37c664".parse().unwrap();
        static ref UUID2: Uuid = "9084aec4-2234-4b9b-8a5d-aac914127255".parse().unwrap();
        static ref UUID3: Uuid = "be6ddf09-5c16-4be4-84dd-d03586eb1fc3".parse().unwrap();
        static ref UUID4: Uuid = "390ef9ab-ce0e-468e-977d-65873274c448".parse().unwrap();
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
            state: ActionStateAgent::New,
            state_payload: None,
        }
    }

    fn mock_core_action(id: Uuid, finished: bool) -> CoreAction {
        let created_ts = Utc::now();
        let finished_ts = if finished { Some(Utc::now()) } else { None };
        let scheduled_ts = finished_ts.clone();
        CoreAction {
            action_id: id,
            args: json!({}),
            cluster_id: "cluster".into(),
            created_ts,
            finished_ts,
            headers: HashMap::new(),
            kind: "action".into(),
            node_id: "node".into(),
            refresh_id: 4321,
            requester: ActionRequester::CoreApi,
            schedule_attempt: 0,
            scheduled_ts,
            state: ActionStateCore::New,
            state_payload: None,
        }
    }

    #[test]
    fn check_ids_to_sync() {
        let store = PrimaryStoreMock::default();
        let a2 = mock_core_action(*UUID2, false);
        // Mock some actions and release the lock.
        {
            let mut store = store.state.lock().expect("MockStore state lock poisoned");
            let a3 = mock_core_action(*UUID3, true);
            store.actions.insert(
                (a2.cluster_id.clone(), a2.node_id.clone(), a2.action_id),
                a2.clone(),
            );
            store.actions.insert(
                (a3.cluster_id.clone(), a3.node_id.clone(), a3.action_id),
                a3,
            );
        }

        // Use the fetcher to check which IDs need to be synced.
        let stream = EventsStream::mock();
        let fetcher =
            ActionsFetcher::new(stream, store.clone().store(), Logger::root(Discard, o!()));
        let remote_ids = vec![*UUID1, *UUID2, *UUID3, *UUID4];
        let (tracer, _) = NoopTracer::new();
        let mut span = tracer.span("test");
        let sync_ids = fetcher
            .check_ids_to_sync("cluster", "node", remote_ids, &mut span)
            .expect("check ids to sync failed");
        assert_eq!(
            sync_ids,
            vec![
                (*UUID1, ActionSyncState::NotFound),
                (*UUID2, ActionSyncState::Found(a2)),
            ],
        );
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
        let store = PrimaryStoreMock::default().store();
        let stream = EventsStream::mock();
        let fetcher = ActionsFetcher::new(stream, store, Logger::root(Discard, o!()));
        let (tracer, _) = NoopTracer::new();
        let mut span = tracer.span("test");
        let ids = fetcher
            .remote_ids(&client, "node", &mut span)
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
        let store = PrimaryStoreMock::default().store();
        let stream = EventsStream::mock();
        let fetcher = ActionsFetcher::new(stream, store, Logger::root(Discard, o!()));
        let (tracer, _) = NoopTracer::new();
        let mut span = tracer.span("test");
        let ids = fetcher
            .remote_ids(&client, "node", &mut span)
            .expect("failed to fetch ids");
        assert_eq!(ids, vec![*UUID3, *UUID2, *UUID1, *UUID4]);
    }

    #[test]
    fn mark_lost_actions() {
        let store = PrimaryStoreMock::default();
        // Mock some actions and release the lock.
        {
            let mut store = store.state.lock().expect("MockStore state lock poisoned");
            let a1 = mock_core_action(*UUID1, false);
            let mut a2 = mock_core_action(*UUID2, false);
            a2.refresh_id = 1234;
            store.actions.insert(
                (a1.cluster_id.clone(), a1.node_id.clone(), a1.action_id),
                a1,
            );
            store.actions.insert(
                (a2.cluster_id.clone(), a2.node_id.clone(), a2.action_id),
                a2,
            );
        }

        // Set up fetcher and run mark function.
        let stream = EventsStream::mock();
        let fetcher =
            ActionsFetcher::new(stream, store.clone().store(), Logger::root(Discard, o!()));
        let (tracer, _) = NoopTracer::new();
        let refresh_id = 1234;
        let mut span = tracer.span("test");
        fetcher
            .mark_lost_actions("cluster", "node", refresh_id, &mut span)
            .expect("marking lost actions failed");

        // Assert actions are lost.
        let action1 = {
            let store = store
                .state
                .lock()
                .expect("MockStore state lock is poisoned");
            let key = ("cluster".into(), "node".into(), *UUID1);
            store.actions.get(&key).expect("action not found").clone()
        };
        let action2 = {
            let store = store
                .state
                .lock()
                .expect("MockStore state lock is poisoned");
            let key = ("cluster".into(), "node".into(), *UUID2);
            store.actions.get(&key).expect("action not found").clone()
        };
        assert_eq!(action1.state, ActionStateCore::Lost);
        assert!(action1.finished_ts.is_some());
        assert_eq!(action2.state, ActionStateCore::New);
        assert!(action2.finished_ts.is_none());
    }

    #[test]
    fn sync_action() {
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
        let store = PrimaryStoreMock::default();
        let stream = EventsStream::mock();
        let fetcher =
            ActionsFetcher::new(stream, store.clone().store(), Logger::root(Discard, o!()));
        let (tracer, _) = NoopTracer::new();
        let refresh_id = 1234;
        let action = CoreAction::new("cluster", "node", refresh_id, action);
        let mut span = tracer.span("test");
        fetcher
            .sync_action(
                &client,
                "cluster",
                "node",
                (*UUID1, ActionSyncState::Found(action)),
                refresh_id,
                &mut span,
            )
            .expect("action sync failed");

        // Assert sync result.
        let action = {
            let store = store.state.lock().expect("MockStore state lock poisoned");
            store
                .actions
                .get(&("cluster".into(), "node".into(), *UUID1))
                .expect("expected action not found")
                .clone()
        };
        assert_eq!(action.action_id, *UUID1);
        assert_eq!(action.state, ActionStateCore::New);
    }

    #[test]
    fn sync_action_not_found() {
        // Set up client.
        let client = MockClient::new(
            || panic!("unused in these tests"),
            || panic!("unused in these tests"),
            || panic!("unused in these tests"),
        );
        let mut action = mock_agent_action(*UUID1, false);
        action.state = ActionStateAgent::New;

        // Set up fetcher and sync an action.
        let store = PrimaryStoreMock::default();
        let stream = EventsStream::mock();
        let fetcher =
            ActionsFetcher::new(stream, store.clone().store(), Logger::root(Discard, o!()));
        let (tracer, _) = NoopTracer::new();
        let refresh_id = 1234;
        let mut span = tracer.span("test");
        fetcher
            .sync_action(
                &client,
                "cluster",
                "node",
                (*UUID1, ActionSyncState::NotFound),
                refresh_id,
                &mut span,
            )
            .expect("action sync failed");

        // Assert sync result.
        let found = {
            let store = store.state.lock().expect("MockStore state lock poisoned");
            store
                .actions
                .get(&("cluster".into(), "node".into(), *UUID1))
                .is_some()
        };
        assert!(!found, "should not have action");
    }
}
