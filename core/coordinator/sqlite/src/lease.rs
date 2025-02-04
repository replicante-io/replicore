//! Lease backend implementation.
use std::time::Duration;

use anyhow::Result;
use tokio_rusqlite::Connection;

use replicore_context::Context;
use replicore_coordinator::ILease;
use replicore_coordinator::State;

/// SQLite backed lease.
pub struct LeaseBackend {
    /// Connection to the SQLite DB.
    connection: Connection,

    /// Unique identifier of the lease.
    id: String,

    /// Last known state of the lease.
    ///
    /// This is used to determine how the lease should be managed (for example acquire vs renew)
    /// as well as lease related events (such as lost leases).
    last_state: State,

    /// Time to live before leases are either renewed or lost.
    ttl: Duration,

    /// Random value for the "lease instance" to check if we are the current owner of the lease.
    value: String,

    /// Interval between DB checks and renewal attempts.
    ///
    /// Must be less or equal to half the TTL value.
    watch_interval: Duration,
}

impl LeaseBackend {
    pub fn new<S>(
        id: S,
        ttl: Duration,
        watch_interval: Duration,
        connection: Connection,
    ) -> LeaseBackend
    where
        S: Into<String>,
    {
        let value = uuid::Uuid::new_v4().to_string();
        let max_watch_interval = ttl / 2;
        LeaseBackend {
            connection,
            id: id.into(),
            last_state: State::Idle,
            ttl,
            value,
            watch_interval: std::cmp::min(watch_interval, max_watch_interval),
        }
    }
}

impl LeaseBackend {
    /// Attempt to acquire ownership of the lease.
    async fn acquire(&mut self, context: &Context) -> Result<State> {
        // Attempt to acquire the lease in the DB.
        let owner_value = crate::statements::acquire(
            context,
            &self.connection,
            self.id.clone(),
            self.value.clone(),
            self.ttl,
        )
        .await?;

        // Check the record persisted in the DB.
        if owner_value == self.value {
            slog::debug!(context.logger, "SQL lease is now PRIMARY"; "lease-id" => &self.id);
            self.last_state = State::Primary;
            Ok(State::Primary)
        } else {
            slog::debug!(context.logger, "SQL lease is now SECONDARY"; "lease-id" => &self.id);
            self.last_state = State::Secondary;
            Ok(State::Secondary)
        }
    }

    /// Loop to renew an owned lease, watch for changes from the backend or acquire lost leases.
    async fn renew(&mut self, context: &Context) -> Result<State> {
        loop {
            tokio::time::sleep(self.watch_interval).await;
            match self.last_state {
                State::Primary => {
                    let renewed = crate::statements::renew_primary(
                        context,
                        &self.connection,
                        self.id.clone(),
                        self.value.clone(),
                        self.ttl,
                    )
                    .await?;
                    if renewed {
                        continue;
                    }
                    self.last_state = State::Lost;
                    return Ok(self.last_state);
                }
                State::Secondary => {
                    let expired_or_lost = crate::statements::expired_or_lost(
                        context,
                        &self.connection,
                        self.id.clone(),
                    )
                    .await?;
                    if expired_or_lost {
                        return self.acquire(context).await;
                    }
                }

                // We can't reach this branch if we are neither primary nor secondary.
                // But just in case return the last state and let the `watch` method handle it.
                _ => return Ok(self.last_state),
            }
        }
    }

    /// Implement [`ILease::watch`] logic when the process is a candidate leaseholder.
    async fn watch_candidate(&mut self, context: &Context) -> Result<State> {
        match self.last_state {
            State::Candidate | State::Lost | State::Idle => self.acquire(context).await,
            State::Primary | State::Secondary => self.renew(context).await,
        }
    }

    /// Implement [`ILease::watch`] logic when the process is NOT a candidate leaseholder.
    async fn watch_not_candidate(&mut self, context: &Context) -> Result<State> {
        match self.last_state {
            State::Primary => {
                self.step_down(context).await?;
                Ok(self.last_state)
            }
            // If the lease is idle and we are not a candidate wait forever.
            // The lease control loop will drop this watch and re-start us when we are candidates.
            State::Idle => std::future::pending().await,
            _ => {
                self.last_state = State::Idle;
                Ok(self.last_state)
            }
        }
    }
}

#[async_trait::async_trait]
impl ILease for LeaseBackend {
    async fn step_down(&mut self, context: &Context) -> Result<()> {
        crate::statements::step_down(
            context,
            &self.connection,
            self.id.clone(),
            self.value.clone(),
        )
        .await?;
        self.last_state = State::Idle;
        Ok(())
    }

    async fn watch(&mut self, context: &Context, candidate: bool) -> Result<State> {
        match candidate {
            true => self.watch_candidate(context).await,
            false => self.watch_not_candidate(context).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use replicore_context::Context;
    use replicore_coordinator::ILease;
    use replicore_coordinator::State;

    use super::LeaseBackend;

    const DEFAULT_TTL: Duration = Duration::from_secs(60);
    const DEFAULT_WATCH_INTERVAL: Duration = Duration::from_millis(2);
    const LEASE_ID: &str = "sqlease";

    /// Initialise an in-memory DB for tests.
    async fn init_db(context: &Context) -> tokio_rusqlite::Connection {
        let connection = crate::factory::create_client(&context, crate::factory::MEMORY_PATH)
            .await
            .expect("DB open failed");
        connection
            .call(move |connection| {
                crate::schema::migrations::runner()
                    .set_migration_table_name(crate::factory::REFINERY_SCHEMA_TABLE_NAME)
                    .run(connection)
                    .expect("DB init failed");
                Ok(())
            })
            .await
            .expect("DB init failed");
        connection
    }

    #[tokio::test]
    async fn acquire() {
        let context = Context::fixture();
        let connection = init_db(&context).await;
        let id = LEASE_ID;
        let ttl = DEFAULT_TTL;
        let mut lease = LeaseBackend::new(id, ttl, DEFAULT_WATCH_INTERVAL, connection.clone());

        let state = lease
            .watch(&context, true)
            .await
            .expect("SQLease to return a state");
        assert_eq!(state, State::Primary);

        // Check lease record from the DB.
        connection
            .call(move |connection| {
                let mut statement = connection
                    .prepare_cached("SELECT * FROM coordinator_lease WHERE lease_id = ?1;")?;
                let mut rows = statement.query([LEASE_ID])?;
                let row = rows.next()?.expect("no lease record found");
                let value: String = row.get("owner_value").unwrap();
                let last_renew: i64 = row.get("last_renew").unwrap();
                let expired_after: i64 = row.get("expired_after").unwrap();

                assert_eq!(value, lease.value);
                assert!(last_renew < expired_after);
                Ok(())
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn acquire_expired() {
        let context = Context::fixture();
        let connection = init_db(&context).await;
        let id = LEASE_ID;
        let ttl = Duration::from_millis(5);

        // Insert an expired lease record.
        connection
            .call(move |connection| {
                let last_renew = time::OffsetDateTime::now_utc() - Duration::from_secs(10);
                let expired_after = last_renew - Duration::from_secs(5);
                let last_renew = last_renew.unix_timestamp();
                let expired_after = expired_after.unix_timestamp();

                connection.execute(
                    r#"
INSERT INTO coordinator_lease (lease_id, owner_value, last_renew, expired_after)
VALUES (?1, ?2, ?3, ?4);"#,
                    rusqlite::params![LEASE_ID, "unit-test", last_renew, expired_after],
                )?;
                Ok(())
            })
            .await
            .unwrap();

        // Acquire the lease and check it is primary.
        let mut lease = LeaseBackend::new(id, ttl, DEFAULT_WATCH_INTERVAL, connection.clone());
        let state = lease
            .watch(&context, true)
            .await
            .expect("SQLease to return a state");
        assert_eq!(state, State::Primary);

        // Check lease record from the DB.
        connection
            .call(move |connection| {
                let mut statement = connection
                    .prepare_cached("SELECT * FROM coordinator_lease WHERE lease_id = ?1;")?;
                let mut rows = statement.query([LEASE_ID])?;
                let row = rows.next()?.expect("no lease record found");
                let value: String = row.get("owner_value").unwrap();

                assert_eq!(value, lease.value);
                Ok(())
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn acquire_secondary() {
        let context = Context::fixture();
        let connection = init_db(&context).await;
        let id = LEASE_ID;
        let ttl = DEFAULT_TTL;

        // Acquire the lease once.
        let mut lease_a = LeaseBackend::new(id, ttl, DEFAULT_WATCH_INTERVAL, connection.clone());
        let state = lease_a
            .watch(&context, true)
            .await
            .expect("SQLease to return a state");
        assert_eq!(state, State::Primary);

        // Attempt to acquire the lease again.
        let mut lease_b = LeaseBackend::new(id, ttl, DEFAULT_WATCH_INTERVAL, connection.clone());
        let state = lease_b
            .watch(&context, true)
            .await
            .expect("SQLease to return a state");
        assert_eq!(state, State::Secondary);

        // Check lease record from the DB.
        connection
            .call(move |connection| {
                let mut statement = connection
                    .prepare_cached("SELECT * FROM coordinator_lease WHERE lease_id = ?1;")?;
                let mut rows = statement.query([LEASE_ID])?;
                let row = rows.next()?.expect("no lease record found");
                let value: String = row.get("owner_value").unwrap();

                assert_eq!(value, lease_a.value);
                Ok(())
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn renew_primary() {
        let context = Context::fixture();
        let connection = init_db(&context).await;
        let id = LEASE_ID;
        let ttl = Duration::from_secs(4);
        let watch_interval = Duration::from_millis(50);

        // Create and acquire the lease.
        let mut lease = LeaseBackend::new(id, ttl, watch_interval, connection.clone());
        let state = lease
            .watch(&context, true)
            .await
            .expect("SQLease to return a state");
        assert_eq!(state, State::Primary);

        // Grab current renew and expiry times.
        let (renewal, expiry) = connection
            .call(move |connection| {
                let mut statement = connection
                    .prepare_cached("SELECT * FROM coordinator_lease WHERE lease_id = ?1;")?;
                let mut rows = statement.query([LEASE_ID])?;
                let row = rows.next()?.expect("no lease record found");
                let renewal: i64 = row.get("last_renew").unwrap();
                let expiry: i64 = row.get("expired_after").unwrap();
                Ok((renewal, expiry))
            })
            .await
            .unwrap();

        // Watch the lease for a while so it can auto-renew.
        let state =
            tokio::time::timeout(Duration::from_millis(1100), lease.watch(&context, true)).await;
        assert!(state.is_err(), "lease watching did not time out");

        // Check lease record from the DB.
        connection
            .call(move |connection| {
                let mut statement = connection
                    .prepare_cached("SELECT * FROM coordinator_lease WHERE lease_id = ?1;")?;
                let mut rows = statement.query([LEASE_ID])?;
                let row = rows.next()?.expect("no lease record found");
                let value: String = row.get("owner_value").unwrap();
                let new_renewal: i64 = row.get("last_renew").unwrap();
                let new_expiry: i64 = row.get("expired_after").unwrap();

                assert_eq!(value, lease.value);
                assert!(new_expiry > expiry);
                assert!(new_renewal > renewal);
                Ok(())
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn renew_primary_not_found() {
        let context = Context::fixture();
        let connection = init_db(&context).await;
        let id = LEASE_ID;
        let ttl = Duration::from_secs(2);
        let watch_interval = Duration::from_millis(10);

        // Create and acquire the lease.
        let mut lease = LeaseBackend::new(id, ttl, watch_interval, connection.clone());
        let state = lease
            .watch(&context, true)
            .await
            .expect("SQLease to return a state");
        assert_eq!(state, State::Primary);

        // Delete the lease record.
        connection
            .call(move |connection| {
                connection.execute(
                    "DELETE FROM coordinator_lease WHERE lease_id = ?1;",
                    rusqlite::params![LEASE_ID],
                )?;
                Ok(())
            })
            .await
            .unwrap();

        // Watch the lease for a while so it can auto-renew.
        let state = tokio::time::timeout(Duration::from_millis(100), lease.watch(&context, true))
            .await
            .expect("lease watch not to time out")
            .expect("lease watch to return a state");
        assert_eq!(state, State::Lost);
    }

    #[tokio::test]
    async fn renew_secondary() {
        let context = Context::fixture();
        let connection = init_db(&context).await;
        let id = LEASE_ID;
        let ttl = Duration::from_secs(4);
        let watch_interval = Duration::from_millis(10);

        // Insert a lease record to be primary.
        connection
            .call(move |connection| {
                let last_renew = time::OffsetDateTime::now_utc() - Duration::from_secs(2);
                let expired_after = last_renew + Duration::from_secs(60);
                let last_renew = last_renew.unix_timestamp();
                let expired_after = expired_after.unix_timestamp();

                connection.execute(
                    r#"
INSERT INTO coordinator_lease (lease_id, owner_value, last_renew, expired_after)
VALUES (?1, ?2, ?3, ?4);"#,
                    rusqlite::params![LEASE_ID, "unit-test", last_renew, expired_after],
                )?;
                Ok(())
            })
            .await
            .unwrap();

        // Create the secondary lease.
        let mut lease = LeaseBackend::new(id, ttl, watch_interval, connection.clone());
        let state = lease
            .watch(&context, true)
            .await
            .expect("SQLease to return a state");
        assert_eq!(state, State::Secondary);

        // Watch the lease for a while so it can auto-renew.
        let state =
            tokio::time::timeout(Duration::from_millis(70), lease.watch(&context, true)).await;
        assert!(state.is_err(), "lease watching did not time out");

        // Check lease record from the DB.
        connection
            .call(move |connection| {
                let mut statement = connection
                    .prepare_cached("SELECT * FROM coordinator_lease WHERE lease_id = ?1;")?;
                let mut rows = statement.query(["sqlease"])?;
                let row = rows.next()?.expect("no lease record found");
                let value: String = row.get("owner_value").unwrap();

                assert_eq!(value, "unit-test");
                Ok(())
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn renew_secondary_expired() {
        let context = Context::fixture();
        let connection = init_db(&context).await;
        let id = LEASE_ID;
        let ttl = Duration::from_secs(4);
        let watch_interval = Duration::from_millis(10);

        // Insert a lease record to be primary.
        connection
            .call(move |connection| {
                let last_renew = time::OffsetDateTime::now_utc() - Duration::from_secs(20);
                let expired_after = last_renew + Duration::from_secs(20);
                let last_renew = last_renew.unix_timestamp();
                let expired_after = expired_after.unix_timestamp();

                connection.execute(
                    r#"
INSERT INTO coordinator_lease (lease_id, owner_value, last_renew, expired_after)
VALUES (?1, ?2, ?3, ?4)
;"#,
                    rusqlite::params!["sqlease", "unit-test", last_renew, expired_after],
                )?;
                Ok(())
            })
            .await
            .unwrap();

        // Create the secondary lease.
        let mut lease = LeaseBackend::new(id, ttl, watch_interval, connection.clone());
        let state = lease
            .watch(&context, true)
            .await
            .expect("SQLease to return a state");
        assert_eq!(state, State::Secondary);

        // Update the lease to be expired.
        connection
            .call(move |connection| {
                let expired_after = time::OffsetDateTime::now_utc() - Duration::from_secs(5);
                let expired_after = expired_after.unix_timestamp();

                connection.execute(
                    r#"
UPDATE coordinator_lease
SET expired_after = ?2
WHERE lease_id = ?1
;"#,
                    rusqlite::params!["sqlease", expired_after],
                )?;
                Ok(())
            })
            .await
            .unwrap();

        // Watch the lease and make sure it becomes primary.
        let state = tokio::time::timeout(Duration::from_millis(70), lease.watch(&context, true))
            .await
            .expect("lease watch not to time out")
            .expect("lease watch to return a state");
        assert_eq!(state, State::Primary);

        // Check lease record from the DB.
        connection
            .call(move |connection| {
                let mut statement = connection
                    .prepare_cached("SELECT * FROM coordinator_lease WHERE lease_id = ?1;")?;
                let mut rows = statement.query(["sqlease"])?;
                let row = rows.next()?.expect("no lease record found");
                let value: String = row.get("owner_value").unwrap();

                assert_eq!(value, lease.value);
                Ok(())
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn renew_secondary_lost() {
        let context = Context::fixture();
        let connection = init_db(&context).await;
        let id = LEASE_ID;
        let ttl = Duration::from_secs(4);
        let watch_interval = Duration::from_millis(10);

        // Insert a lease record to be primary.
        connection
            .call(move |connection| {
                let last_renew = time::OffsetDateTime::now_utc() - Duration::from_secs(20);
                let expired_after = last_renew + Duration::from_secs(20);
                let last_renew = last_renew.unix_timestamp();
                let expired_after = expired_after.unix_timestamp();

                connection.execute(
                    r#"
INSERT INTO coordinator_lease (lease_id, owner_value, last_renew, expired_after)
VALUES (?1, ?2, ?3, ?4)
;"#,
                    rusqlite::params!["sqlease", "unit-test", last_renew, expired_after],
                )?;
                Ok(())
            })
            .await
            .unwrap();

        // Create the secondary lease.
        let mut lease = LeaseBackend::new(id, ttl, watch_interval, connection.clone());
        let state = lease
            .watch(&context, true)
            .await
            .expect("SQLease to return a state");
        assert_eq!(state, State::Secondary);

        // Update the lease to be expired.
        connection
            .call(move |connection| {
                connection.execute(
                    "DELETE FROM coordinator_lease WHERE lease_id = ?1;",
                    rusqlite::params!["sqlease"],
                )?;
                Ok(())
            })
            .await
            .unwrap();

        // Watch the lease and make sure it becomes primary.
        let state = tokio::time::timeout(Duration::from_millis(70), lease.watch(&context, true))
            .await
            .expect("lease watch not to time out")
            .expect("lease watch to return a state");
        assert_eq!(state, State::Primary);

        // Check lease record from the DB.
        connection
            .call(move |connection| {
                let mut statement = connection
                    .prepare_cached("SELECT * FROM coordinator_lease WHERE lease_id = ?1;")?;
                let mut rows = statement.query(["sqlease"])?;
                let row = rows.next()?.expect("no lease record found");
                let value: String = row.get("owner_value").unwrap();

                assert_eq!(value, lease.value);
                Ok(())
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn no_can_idle() {
        let context = Context::fixture();
        let connection = init_db(&context).await;
        let id = LEASE_ID;
        let ttl = DEFAULT_TTL;

        // Create lease.
        let mut lease = LeaseBackend::new(id, ttl, DEFAULT_WATCH_INTERVAL, connection.clone());
        let state = lease
            .watch(&context, true)
            .await
            .expect("SQLease to return a state");
        assert_eq!(state, State::Primary);

        // Stop being candidate and move to idle.
        let state = lease
            .watch(&context, false)
            .await
            .expect("SQLease to return a state");
        assert_eq!(state, State::Idle);

        // Ensure it waits "forever" once in idle.
        let state =
            tokio::time::timeout(Duration::from_millis(70), lease.watch(&context, false)).await;
        assert!(state.is_err(), "lease watching did not time out");
    }

    #[tokio::test]
    async fn no_can_primary() {
        let context = Context::fixture();
        let connection = init_db(&context).await;
        let id = LEASE_ID;
        let ttl = DEFAULT_TTL;
        let mut lease = LeaseBackend::new(id, ttl, DEFAULT_WATCH_INTERVAL, connection.clone());

        let state = lease
            .watch(&context, true)
            .await
            .expect("SQLease to return a state");
        assert_eq!(state, State::Primary);

        let state = lease
            .watch(&context, false)
            .await
            .expect("SQLease to return a state");
        assert_eq!(state, State::Idle);

        // Check lease record from the DB.
        connection
            .call(move |connection| {
                let mut statement = connection
                    .prepare_cached("SELECT * FROM coordinator_lease WHERE lease_id = ?1;")?;
                let mut rows = statement.query(["sqlease"])?;
                let row = rows.next()?;

                assert!(row.is_none(), "lease record not deleted");
                Ok(())
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn no_can_secondary() {
        let context = Context::fixture();
        let connection = init_db(&context).await;
        let id = LEASE_ID;
        let ttl = DEFAULT_TTL;

        // Insert a lease record to be primary.
        connection
            .call(move |connection| {
                let last_renew = time::OffsetDateTime::now_utc() - Duration::from_secs(20);
                let expired_after = last_renew + Duration::from_secs(20);
                let last_renew = last_renew.unix_timestamp();
                let expired_after = expired_after.unix_timestamp();

                connection.execute(
                    r#"
INSERT INTO coordinator_lease (lease_id, owner_value, last_renew, expired_after)
VALUES (?1, ?2, ?3, ?4)
;"#,
                    rusqlite::params!["sqlease", "unit-test", last_renew, expired_after],
                )?;
                Ok(())
            })
            .await
            .unwrap();

        let mut lease = LeaseBackend::new(id, ttl, DEFAULT_WATCH_INTERVAL, connection.clone());
        let state = lease
            .watch(&context, true)
            .await
            .expect("SQLease to return a state");
        assert_eq!(state, State::Secondary);

        let state = lease
            .watch(&context, false)
            .await
            .expect("SQLease to return a state");
        assert_eq!(state, State::Idle);

        // Check lease record from the DB.
        connection
            .call(move |connection| {
                let mut statement = connection
                    .prepare_cached("SELECT * FROM coordinator_lease WHERE lease_id = ?1;")?;
                let mut rows = statement.query(["sqlease"])?;
                let row = rows.next()?.expect("no lease record found");
                let value: String = row.get("owner_value").unwrap();

                assert_eq!(value, "unit-test");
                Ok(())
            })
            .await
            .unwrap();
    }
}
