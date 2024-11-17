use super::fixture::LeaseFixture;
use super::fixture::LeaseFixtureNotification;
use super::Lease;
use super::State;

#[tokio::test]
async fn cancel_task_on_drop() {
    let lease = LeaseFixture::fixed(State::Secondary);
    let notifs = lease.notifications();
    let lease = Lease::new("test", lease);
    drop(lease);

    let messages = notifs.snapshot();
    assert_eq!(messages, vec![]);
}

#[tokio::test]
async fn step_down_happens() {
    let lease = LeaseFixture::fixed(State::Primary);
    let notifs = lease.notifications();
    let mut lease = Lease::new("test", lease);

    // Check watched state.
    let state = lease.watch().await.unwrap();
    assert_eq!(state, State::Primary);

    // Step down and watch the state again.
    let delay = std::time::Duration::from_secs(1);
    lease.step_down(delay).await.unwrap();
    let state = lease.watch().await.unwrap();
    assert_eq!(state, State::Lost);

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
    let lease =
        LeaseFixture::fixed(State::Primary).set_watch_delay(std::time::Duration::from_millis(9));
    let notifs = lease.notifications();
    let mut lease = Lease::new("test", lease);

    // Check watched state.
    let state = lease.watch().await.unwrap();
    assert_eq!(state, State::Primary);

    // Step down and watch the state again.
    let delay = std::time::Duration::from_millis(10);
    lease.step_down(delay).await.unwrap();
    let state = lease.watch().await.unwrap();
    assert_eq!(state, State::Lost);

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
    let lease = LeaseFixture::fixed(State::Primary);
    let notifs = lease.notifications();
    let mut lease = Lease::new("test", lease);

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
    let lease = LeaseFixture::simple_transition(State::Primary, State::Secondary, 3);
    let notifs = lease.notifications();
    let mut lease = Lease::new("test", lease);

    // Check watched state.
    let state = lease.watch().await.unwrap();
    assert_eq!(state, State::Primary);
    let state = lease.watch().await.unwrap();
    assert_eq!(state, State::Lost);
    let state = lease.watch().await.unwrap();
    assert_eq!(state, State::Secondary);
    drop(lease);

    // Confirm the stab was watched 5 times (2 Primary + 2 Secondary).
    let messages = notifs.snapshot();
    assert_eq!(
        messages,
        vec![
            LeaseFixtureNotification::Watched(State::Primary),
            LeaseFixtureNotification::Watched(State::Primary),
            LeaseFixtureNotification::Watched(State::Primary),
            LeaseFixtureNotification::Watched(State::Secondary),
            LeaseFixtureNotification::Watched(State::Secondary),
        ]
    );
}
