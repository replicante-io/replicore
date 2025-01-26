# SQLite Coordinator backend

SQLite backends are intended for small, single-process instances.
For coordination, the SQLite backend was preferred over an in-memory option because:

1. A SQLite implementation provides a foundation for a later SQL-generic backend.
   Such backends would naturally adapt to both small scale (SQLite backed) core clusters
   as well as multi-node (PostgreSQL backed) core clusters.
2. Using a SQLite DB allows lease inspection by connecting to the SQLite DB while core is running.

## How leases work

Leases are stored in a table with the following attributes:

- Lease ID: unique ID so multiple threads can correctly handle exclusive logic.
- Ownership Value: random value ensure only the lease owner can perform changes to the lease.
  This is used in for compare and swap style operations.
- Last renew: timestamp of the latest lease renewal update.
- Expired after: timestamp after which the lease is lost/expired, if not renewed first.

Lease operations are implemented in the simplest possible way and with "atomic properties"
(so if a statement succeeds the lease it acquired/renewed):

- Acquire: is implemented as an SQL insert. Lease ID duplication means we can't acquire the lease.
  If possible, an `ON CONFLICT` clause is used to acquire in the presence expired leases.
- Renewal: is something along the lines of `UPDATE ... WHERE id = ... AND value = ... AND expire < now()`.
  The expiry time is also checked to ensure an expired lease is not renewed without a re-election.
  Possibly a `RETURNING` element is also included so the renewal can be confirmed client side.
- Detecting loss of lease: a lease is lost when it was owned and a renewal operation fails.
- Stepping down: is implemented as an SQL delete.

Finally, expired leases are periodically purged from the database.
This ensures the DB remains small and clean over time.
