use std::collections::HashSet;

use failure::ResultExt;
use opentracingrust::Span;
use uuid::Uuid;

use replicante_agent_client::Client;
use replicante_models_core::actions::Action;
use replicante_store_primary::store::actions::ActionSyncState;
use replicante_store_primary::store::actions::MAX_ACTIONS_STATE_FOR_SYNC;
use replicante_store_primary::store::Store;

use crate::metrics::FETCHER_ACTIONS_CHUNKS;
use crate::metrics::FETCHER_ACTIONS_SYNCED;
use crate::ErrorKind;
use crate::Result;

/// Actions fetch and sync processing.
pub(crate) struct ActionsFetcher {
    store: Store,
}

impl ActionsFetcher {
    pub fn new(store: Store) -> ActionsFetcher {
        ActionsFetcher { store }
    }

    /// Sync the actions for the given node.
    pub fn sync(
        &self,
        client: &dyn Client,
        cluster_id: &str,
        node_id: &str,
        refresh_id: i64,
        span: &mut Span,
    ) -> Result<()> {
        let remote_ids = self.remote_ids(client, node_id, span)?;
        let sync_ids = self.check_ids_to_sync(cluster_id, node_id, remote_ids, span)?;
        let sync_size = sync_ids.len();
        for action_id in sync_ids {
            self.sync_action(client, cluster_id, node_id, action_id, refresh_id, span)?;
        }
        self.mark_lost_actions(cluster_id, node_id, refresh_id, span)?;
        FETCHER_ACTIONS_SYNCED.observe(sync_size as f64);
        Ok(())
    }

    /// Check the given remote IDs against the primary store and return a list of IDs to sync.
    fn check_ids_to_sync(
        &self,
        cluster_id: &str,
        node_id: &str,
        remote_ids: Vec<Uuid>,
        span: &mut Span,
    ) -> Result<Vec<Uuid>> {
        let mut results = Vec::new();
        let chunks = remote_ids.chunks(MAX_ACTIONS_STATE_FOR_SYNC);
        let chunks_count = chunks.len();
        for ids in chunks {
            let states = self
                .store
                .actions(cluster_id.to_string())
                .state_for_sync(node_id.to_string(), &ids, span.context().clone())
                .with_context(|_| ErrorKind::StoreRead("actions state for sync"))?;
            for id in ids {
                let state = states
                    .get(id)
                    .expect("state_for_sync did not return all IDs");
                match state {
                    // Once we find the first `Finished` action we stop checking.
                    // This works because of required ordering of actions.
                    // See bin/replicante/src/tasks/cluster_refresh/mod.rs for details on the sync process.
                    ActionSyncState::Finished => return Ok(results),
                    _ => results.push(*id),
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
        self.store
            .actions(cluster_id.to_string())
            .mark_lost(node_id.to_string(), refresh_id, span.context().clone())
            .with_context(|_| ErrorKind::StoreWrite("mark actions as lost"))?;
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

    /// Sync a single action's details.
    fn sync_action(
        &self,
        client: &dyn Client,
        cluster_id: &str,
        node_id: &str,
        action_id: Uuid,
        refresh_id: i64,
        span: &mut Span,
    ) -> Result<()> {
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
        self.store
            .persist()
            .action(action, span.context().clone())
            .with_context(|_| ErrorKind::StoreWrite("action"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::Utc;
    use opentracingrust::tracers::NoopTracer;
    use serde_json::json;
    use uuid::Uuid;

    use replicante_agent_client::mock::MockClient;
    use replicante_models_agent::actions::api::ActionInfoResponse;
    use replicante_models_agent::actions::ActionListItem;
    use replicante_models_agent::actions::ActionModel as AgentActionModel;
    use replicante_models_agent::actions::ActionState as ActionStateAgent;
    use replicante_models_core::actions::Action as CoreAction;
    use replicante_models_core::actions::ActionRequester;
    use replicante_models_core::actions::ActionState as ActionStateCore;
    use replicante_store_primary::mock::Mock;

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
        AgentActionModel {
            args: json!({}),
            created_ts,
            finished_ts,
            headers: HashMap::new(),
            id,
            kind: "action".into(),
            requester: ActionRequester::Api,
            state: ActionStateAgent::New,
            state_payload: None,
        }
    }

    fn mock_core_action(id: Uuid, finished: bool) -> CoreAction {
        let created_ts = Utc::now();
        let finished_ts = if finished { Some(Utc::now()) } else { None };
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
            requester: ActionRequester::Api,
            state: ActionStateCore::New,
            state_payload: None,
        }
    }

    #[test]
    fn check_ids_to_sync() {
        let store = Mock::default();
        // Mock some actions and release the lock.
        {
            let mut store = store.state.lock().expect("MockStore state lock poisoned");
            let a2 = mock_core_action(*UUID2, false);
            let a3 = mock_core_action(*UUID3, true);
            store.actions.insert(
                (a2.cluster_id.clone(), a2.node_id.clone(), a2.action_id),
                a2,
            );
            store.actions.insert(
                (a3.cluster_id.clone(), a3.node_id.clone(), a3.action_id),
                a3,
            );
        }

        // Use the fetcher to check which IDs need to be synced.
        let fetcher = ActionsFetcher::new(store.clone().store());
        let remote_ids = vec![*UUID1, *UUID2, *UUID3, *UUID4];
        let (tracer, _) = NoopTracer::new();
        let mut span = tracer.span("test");
        let sync_ids = fetcher
            .check_ids_to_sync("cluster", "node", remote_ids, &mut span)
            .expect("check ids to sync failed");
        assert_eq!(sync_ids, vec![*UUID1, *UUID2]);
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
        let store = Mock::default().store();
        let fetcher = ActionsFetcher::new(store);
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
        let store = Mock::default().store();
        let fetcher = ActionsFetcher::new(store);
        let (tracer, _) = NoopTracer::new();
        let mut span = tracer.span("test");
        let ids = fetcher
            .remote_ids(&client, "node", &mut span)
            .expect("failed to fetch ids");
        assert_eq!(ids, vec![*UUID3, *UUID2, *UUID1, *UUID4]);
    }

    #[test]
    fn mark_lost_actions() {
        let store = Mock::default();
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
        let fetcher = ActionsFetcher::new(store.clone().store());
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
            action,
            history: Vec::new(),
        };
        client.actions.insert(*UUID1, info);

        // Set up fetcher and sync an action.
        let store = Mock::default();
        let fetcher = ActionsFetcher::new(store.clone().store());
        let (tracer, _) = NoopTracer::new();
        let refresh_id = 1234;
        let mut span = tracer.span("test");
        fetcher
            .sync_action(&client, "cluster", "node", *UUID1, refresh_id, &mut span)
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
        let store = Mock::default();
        let fetcher = ActionsFetcher::new(store.clone().store());
        let (tracer, _) = NoopTracer::new();
        let refresh_id = 1234;
        let mut span = tracer.span("test");
        fetcher
            .sync_action(&client, "cluster", "node", *UUID1, refresh_id, &mut span)
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
