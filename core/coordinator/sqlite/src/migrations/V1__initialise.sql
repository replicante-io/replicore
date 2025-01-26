-- Track all leases known to the system.
CREATE TABLE IF NOT EXISTS coordinator_lease(
  -- Unique ID of the lease, shared by all processes coordinating the same task.
  lease_id TEXT PRIMARY KEY,
  -- Random value set on acquire, to determine who the current owner of the lease is.
  owner_value TEXT NOT NULL,
  -- EPoc timestamp (in seconds) of the most recent renewal operation.
  last_renew INTEGER NOT NULL,
  -- EPoc timestamp (in seconds) after which the lease is considered lost.
  expired_after INTEGER NOT NULL
);
