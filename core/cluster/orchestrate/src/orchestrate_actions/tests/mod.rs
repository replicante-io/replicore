use uuid::Uuid;

use replicante_models_core::actions::orchestrator::OrchestratorAction;
use replicante_models_core::actions::orchestrator::OrchestratorActionState;
use replicante_models_core::events::action::ActionEvent;
use replicante_models_core::events::Event;
use replicante_models_core::events::Payload;

mod fixtures;

fn get_action(fixture: &self::fixtures::FixtureData, uuid: Uuid) -> OrchestratorAction {
    let key = (crate::tests::fixtures::CLUSTER_ID.into(), uuid);
    fixture
        .mock_store
        .state
        .lock()
        .expect("MockState lock poisoned")
        .orchestrator_actions
        .get(&key)
        .cloned()
        .expect("action to be in the store")
}

fn get_action_state_and_count_index(
    fixture: &self::fixtures::FixtureData,
    uuid: Uuid,
) -> (OrchestratorActionState, Option<i64>) {
    let action = get_action(fixture, uuid);
    let count_index = action.state_payload.and_then(|payload| {
        payload
            .get("count_index")
            .and_then(|count_index| count_index.as_i64())
    });
    (action.state, count_index)
}

// Return all events on the events stream after setting their timestamp to 0.
fn get_events(data: &crate::ClusterOrchestrate) -> Vec<Event> {
    data.events
        .short_follow("test", None)
        .expect("events to be returned by stream")
        .map(|message| message.expect("events stream to have valid messages"))
        .map(|message| {
            let mut event = message
                .payload()
                .expect("events stream to have valid messages");
            message.async_ack().expect("message to be acked");
            event.timestamp = chrono::DateTime::from_utc(
                chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                chrono::Utc,
            );
            event
        })
        .collect()
}

/// Set the scheduled_ts value for an action.
fn set_scheduled_ts(
    fixture: &self::fixtures::FixtureData,
    uuid: Uuid,
    scheduled_ts: chrono::DateTime<chrono::Utc>,
) {
    let key = (crate::tests::fixtures::CLUSTER_ID.into(), uuid);
    fixture
        .mock_store
        .state
        .lock()
        .expect("MockState lock poisoned")
        .orchestrator_actions
        .get_mut(&key)
        .expect("expect action to be in the store")
        .scheduled_ts = Some(scheduled_ts);
}

#[test]
fn fail_action_on_handler_error() {
    let mut report = crate::tests::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, fixture) =
        self::fixtures::cluster(&mut report, self::fixtures::cluster_fill_running_fail);

    super::orchestrate(&data, &mut data_mut).expect("cluster to orchestrate");

    // Check actions have progressed.
    let action = get_action(&fixture, *self::fixtures::UUID3);
    assert_eq!(action.state, OrchestratorActionState::Failed);
    assert_eq!(action.state_payload, None);
    assert_eq!(
        action.state_payload_error.clone().unwrap(),
        serde_json::json!({
            "info": {
                "message": "debug action failed intentionally",
            },
            "causes": [],
        })
    );

    // Check failed action event is emitted.
    let events: Vec<Event> = get_events(&data);
    let expected = {
        let mut event = Event::builder()
            .action()
            .orchestrator_action_finished(action);
        event.timestamp = chrono::DateTime::from_utc(
            chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
            chrono::Utc,
        );
        event
    };
    assert_eq!(events, vec![expected]);
}

#[test]
fn fail_action_on_handler_error_with_context() {
    let mut report = crate::tests::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, fixture) =
        self::fixtures::cluster(&mut report, self::fixtures::cluster_fill_running_fail);

    // Patch the failing action to wrap the error.
    let key = (
        crate::tests::fixtures::CLUSTER_ID.into(),
        *self::fixtures::UUID3,
    );
    fixture
        .mock_store
        .state
        .lock()
        .expect("MockState lock poisoned")
        .orchestrator_actions
        .get_mut(&key)
        .expect("expect action to be in the store")
        .args = serde_json::json!({ "wrapped": true });

    super::orchestrate(&data, &mut data_mut).expect("cluster to orchestrate");

    // Check actions have progressed.
    let action = get_action(&fixture, *self::fixtures::UUID3);
    assert_eq!(action.state, OrchestratorActionState::Failed);
    assert_eq!(action.state_payload, None);
    assert_eq!(
        action.state_payload_error.unwrap(),
        serde_json::json!({
            "info": {
                "message": "debug action wrapped in some context",
            },
            "causes": [{
                "message": "debug action failed intentionally",
            }],
        })
    );
}

#[test]
fn fail_action_on_handler_not_found() {
    let mut report = crate::tests::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, fixture) =
        self::fixtures::cluster(&mut report, self::fixtures::cluster_fill_running_missing);

    super::orchestrate(&data, &mut data_mut).expect("cluster to orchestrate");

    // Check actions have progressed.
    let action = get_action(&fixture, *self::fixtures::UUID4);
    assert_eq!(action.state, OrchestratorActionState::Failed);
    assert_eq!(action.state_payload, None);
    assert_eq!(
        action.state_payload_error.clone().unwrap(),
        serde_json::json!({
            "info": {
                "message": format!(
                    "unsupported kind test.core.replicante.io/action.not.found for orchestrator action {} in cluster default.colours",
                    *self::fixtures::UUID4,
                ),
            },
            "causes": [],
        })
    );

    // Check failed action event is emitted.
    let events: Vec<Event> = get_events(&data);
    let expected = {
        let mut event = Event::builder()
            .action()
            .orchestrator_action_finished(action);
        event.timestamp = chrono::DateTime::from_utc(
            chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
            chrono::Utc,
        );
        event
    };
    assert_eq!(events, vec![expected]);
}

#[test]
fn final_states_finish_actions() {
    let mut report = crate::tests::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, fixture) =
        self::fixtures::cluster(&mut report, |cluster_view, store, registry| {
            self::fixtures::cluster_fill_running_fail(cluster_view, store, registry);
            self::fixtures::cluster_fill_running_success(cluster_view, store, registry);
        });

    super::orchestrate(&data, &mut data_mut).expect("cluster to orchestrate");

    // Check actions have progressed.
    let action = get_action(&fixture, *self::fixtures::UUID3);
    assert_eq!(action.state, OrchestratorActionState::Failed);
    assert!(action.finished_ts.is_some());

    let action = get_action(&fixture, *self::fixtures::UUID5);
    assert_eq!(action.state, OrchestratorActionState::Done);
    assert!(action.finished_ts.is_some());

    // Check action finished events were emitted.
    let events: Vec<Event> = get_events(&data);
    let expected_uuid3 = {
        let action = get_action(&fixture, *self::fixtures::UUID3);
        let mut event = Event::builder()
            .action()
            .orchestrator_action_finished(action);
        event.timestamp = chrono::DateTime::from_utc(
            chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
            chrono::Utc,
        );
        event
    };
    let expected_uuid5 = {
        let action = get_action(&fixture, *self::fixtures::UUID5);
        let mut event = Event::builder()
            .action()
            .orchestrator_action_finished(action);
        event.timestamp = chrono::DateTime::from_utc(
            chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
            chrono::Utc,
        );
        event
    };
    assert_eq!(events, vec![expected_uuid3, expected_uuid5]);
}

#[test]
fn progress_all_running_actions() {
    let mut report = crate::tests::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, fixture) =
        self::fixtures::cluster(&mut report, self::fixtures::cluster_fill_running_counts);

    super::orchestrate(&data, &mut data_mut).expect("cluster to orchestrate");

    // Check actions have progressed.
    let (state, count_index) = get_action_state_and_count_index(&fixture, *self::fixtures::UUID1);
    assert_eq!(state, OrchestratorActionState::Running);
    assert_eq!(count_index, Some(3));
    let (state, count_index) = get_action_state_and_count_index(&fixture, *self::fixtures::UUID2);
    assert_eq!(state, OrchestratorActionState::Done);
    assert_eq!(count_index, Some(5));

    // Check progressed actions have a scheduled_ts set.
    let action = get_action(&fixture, *self::fixtures::UUID1);
    assert!(action.scheduled_ts.is_some());

    // Check action changed events were emitted.
    let events: Vec<Event> = get_events(&data);
    assert_eq!(events.len(), 2);
    match &events[0].payload {
        Payload::Action(event) => match event {
            ActionEvent::OrchestratorChanged(record) => {
                assert_eq!(record.current.action_id, *self::fixtures::UUID1);
            }
            _ => panic!("expected action event for changed action"),
        },
        _ => panic!("expected action event"),
    }
    match &events[1].payload {
        Payload::Action(event) => match event {
            ActionEvent::OrchestratorFinished(action) => {
                assert_eq!(action.action_id, *self::fixtures::UUID2);
            }
            _ => panic!("expected action event for changed action"),
        },
        _ => panic!("expected action event"),
    }
}

#[test]
fn start_pending_actions() {
    let mut report = crate::tests::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, fixture) =
        self::fixtures::cluster(&mut report, self::fixtures::cluster_fill_pending_actions);

    super::orchestrate(&data, &mut data_mut).expect("cluster to orchestrate");

    // Check actions have progressed.
    let (state, count_index) = get_action_state_and_count_index(&fixture, *self::fixtures::UUID6);
    assert_eq!(state, OrchestratorActionState::Running);
    assert_eq!(count_index, Some(1));
    let (state, count_index) = get_action_state_and_count_index(&fixture, *self::fixtures::UUID7);
    assert_eq!(state, OrchestratorActionState::PendingApprove);
    assert_eq!(count_index, None);
    let (state, count_index) = get_action_state_and_count_index(&fixture, *self::fixtures::UUID8);
    assert_eq!(state, OrchestratorActionState::PendingSchedule);
    assert_eq!(count_index, None);

    // Check newly started actions have a scheduled_ts set.
    let action = get_action(&fixture, *self::fixtures::UUID6);
    assert!(action.scheduled_ts.is_some());

    // Check action changed events were emitted.
    let events: Vec<Event> = get_events(&data);
    assert_eq!(events.len(), 1);
    match &events[0].payload {
        Payload::Action(event) => match event {
            ActionEvent::OrchestratorChanged(record) => {
                assert_eq!(record.current.action_id, *self::fixtures::UUID6);
            }
            _ => panic!("expected action event for changed action"),
        },
        _ => panic!("expected action event"),
    }
}

#[test]
fn timeout_action_default_for_kind() {
    let mut report = crate::tests::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, fixture) =
        self::fixtures::cluster(&mut report, self::fixtures::cluster_fill_running_counts);

    // Set a scheduled_ts past the default timeout.
    set_scheduled_ts(
        &fixture,
        *self::fixtures::UUID1,
        chrono::Utc::now() - chrono::Duration::weeks(1),
    );
    set_scheduled_ts(
        &fixture,
        *self::fixtures::UUID2,
        chrono::Utc::now() - chrono::Duration::weeks(1),
    );

    super::orchestrate(&data, &mut data_mut).expect("cluster to orchestrate");

    // Check the long-running action failed with timeout.
    let action = get_action(&fixture, *self::fixtures::UUID1);
    assert_eq!(action.state, OrchestratorActionState::Failed);
    assert_eq!(
        action.state_payload_error.clone().unwrap(),
        serde_json::json!({
            "info": {
                "message": format!(
                    "orchestrator action {} in cluster default.colours did not finish in time and was failed",
                    *self::fixtures::UUID1,
                ),
            },
            "causes": [],
        })
    );

    // Check the finished action did not failed.
    let action = get_action(&fixture, *self::fixtures::UUID2);
    assert_eq!(action.state, OrchestratorActionState::Done);
    assert_eq!(action.state_payload_error, None);
}

#[test]
fn timeout_action_from_override() {
    let mut report = crate::tests::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, fixture) =
        self::fixtures::cluster(&mut report, self::fixtures::cluster_fill_running_counts);

    // Set a scheduled_ts past the default timeout.
    set_scheduled_ts(
        &fixture,
        *self::fixtures::UUID1,
        chrono::Utc::now() - chrono::Duration::minutes(30),
    );
    let key = (
        crate::tests::fixtures::CLUSTER_ID.into(),
        *self::fixtures::UUID1,
    );
    fixture
        .mock_store
        .state
        .lock()
        .expect("MockState lock poisoned")
        .orchestrator_actions
        .get_mut(&key)
        .expect("expect action to be in the store")
        .timeout = std::time::Duration::from_secs(60 * 10).into();

    super::orchestrate(&data, &mut data_mut).expect("cluster to orchestrate");

    // Check the long-running action failed with timeout.
    let action = get_action(&fixture, *self::fixtures::UUID1);
    assert_eq!(action.state, OrchestratorActionState::Failed);
    assert_eq!(
        action.state_payload_error.clone().unwrap(),
        serde_json::json!({
            "info": {
                "message": format!(
                    "orchestrator action {} in cluster default.colours did not finish in time and was failed",
                    *self::fixtures::UUID1,
                ),
            },
            "causes": [],
        })
    );
}
