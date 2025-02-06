use replicore_context::Context;

use crate::Lease;
use crate::LeaseFixture;
use crate::LeaseFixtureNotification;
use crate::State;

#[tokio::test]
async fn cancel_task_on_drop() {
    let context = Context::fixture();
    let lease = LeaseFixture::fixed(State::Secondary);
    let notifs = lease.notifications();
    let lease = Lease::new(context, "test", lease);
    drop(lease);

    let messages = notifs.snapshot();
    assert_eq!(messages, vec![]);
}

#[tokio::test]
async fn step_down_happens() {
    let context = Context::fixture();
    let lease = LeaseFixture::fixed(State::Primary);
    let notifs = lease.notifications();
    let mut lease = Lease::new(context, "test", lease);

    // Check watched state.
    let state = lease.watch().await.unwrap();
    assert_eq!(state, State::Primary);

    // Step down and watch the state again.
    let delay = std::time::Duration::from_secs(1);
    lease.step_down(delay).await.unwrap();
    let state = lease.watch().await.unwrap();
    assert_eq!(state, State::Idle);

    // Confirm the stab was watched and stepped down.
    drop(lease);
    let messages = notifs.snapshot();
    assert_eq!(
        messages,
        vec![
            LeaseFixtureNotification::Watched(State::Primary),
            LeaseFixtureNotification::SteppedDown,
            LeaseFixtureNotification::Watched(State::Idle),
        ]
    );
}

#[tokio::test]
async fn step_down_is_temporary() {
    let context = Context::fixture();
    let lease =
        LeaseFixture::fixed(State::Primary).set_watch_delay(std::time::Duration::from_millis(9));
    let notifs = lease.notifications();
    let mut lease = Lease::new(context, "test", lease);

    // Check watched state.
    let state = lease.watch().await.unwrap();
    assert_eq!(state, State::Primary);

    // Step down and watch the state again.
    let delay = std::time::Duration::from_millis(10);
    lease.step_down(delay).await.unwrap();
    let state = lease.watch().await.unwrap();
    assert_eq!(state, State::Idle);

    tokio::time::sleep(delay * 2).await;
    let state = lease.watch().await.unwrap();
    assert_eq!(state, State::Primary);

    // Confirm the stab was watched and stepped down.
    drop(lease);
    let messages = notifs.snapshot();
    assert_eq!(
        messages,
        vec![
            LeaseFixtureNotification::Watched(State::Primary),
            LeaseFixtureNotification::SteppedDown,
            LeaseFixtureNotification::Watched(State::Idle),
            LeaseFixtureNotification::Watched(State::Primary),
        ]
    );
}

#[tokio::test]
async fn watch_returns_state_changes() {
    let context = Context::fixture();
    let lease = LeaseFixture::fixed(State::Primary);
    let notifs = lease.notifications();
    let mut lease = Lease::new(context, "test", lease);

    // Check watched state.
    let state = lease.watch().await.unwrap();
    assert_eq!(state, State::Primary);
    drop(lease);

    // Confirm the stab was watched.
    let messages = notifs.snapshot();
    assert_eq!(
        messages,
        vec![LeaseFixtureNotification::Watched(State::Primary),]
    );
}

#[tokio::test]
async fn watch_filters_same_state_notifications() {
    let context = Context::fixture();
    let lease = LeaseFixture::simple_transition(State::Primary, State::Secondary, 3);
    let notifs = lease.notifications();
    let mut lease = Lease::new(context, "test", lease);

    // Check watched state.
    let state = lease.watch().await.unwrap();
    assert_eq!(state, State::Primary);
    let state = lease.watch().await.unwrap();
    assert_eq!(state, State::Secondary);
    drop(lease);

    // Confirm the stab was watched 4 times (3 Primary + 1 Secondary).
    let messages = notifs.snapshot();
    assert_eq!(
        messages,
        vec![
            LeaseFixtureNotification::Watched(State::Primary),
            LeaseFixtureNotification::Watched(State::Primary),
            LeaseFixtureNotification::Watched(State::Primary),
            LeaseFixtureNotification::Watched(State::Secondary),
        ]
    );
}

#[tokio::test]
async fn lease_handle_updates() {
    let context = Context::fixture();
    let lease = LeaseFixture::fixed(State::Primary);

    let mut lease = Lease::new(context, "test", lease);
    let handle = lease.handle();
    assert_eq!(handle.state(), State::Idle);

    let state = tokio::time::timeout(std::time::Duration::from_millis(100), lease.watch())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(state, State::Primary);
    assert_eq!(handle.state(), State::Primary);

    lease
        .step_down(std::time::Duration::from_secs(2))
        .await
        .unwrap();
    assert_eq!(handle.state(), State::Primary);

    let state = tokio::time::timeout(std::time::Duration::from_millis(100), lease.watch())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(state, State::Idle);
    assert_eq!(handle.state(), State::Idle);
}

#[tokio::test]
async fn lease_handle_step_down() {
    let context = Context::fixture();
    let lease = LeaseFixture::fixed(State::Primary);

    let mut lease = Lease::new(context, "test", lease);
    let handle = lease.handle();
    assert_eq!(handle.state(), State::Idle);

    let state = tokio::time::timeout(std::time::Duration::from_millis(100), lease.watch())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(state, State::Primary);

    handle
        .step_down(std::time::Duration::from_secs(2))
        .await
        .unwrap();
    let state = tokio::time::timeout(std::time::Duration::from_millis(100), lease.watch())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(state, State::Idle);
    assert_eq!(handle.state(), State::Idle);
}
