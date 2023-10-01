-- Events are stored as JSON objects with virtual columns for indexes.
-- Audit and change events are stored in separate tables for performance and fault isolation.

CREATE TABLE IF NOT EXISTS events_audit(
  -- Event as a JSON blob.
  event TEXT NOT NULL,

  -- Manually managed normalised columns for indexes (where advanced types are involved).
  time REAL NOT NULL,

  -- Virtual columns to index events on.
  code TEXT NOT NULL AS (json_extract(event, '$.code'))
);
CREATE INDEX events_audit_time ON events_audit(time, code);

CREATE TABLE IF NOT EXISTS events_change(
  -- Event as a JSON blob.
  event TEXT NOT NULL,

  -- Manually managed normalised columns for indexes (where advanced types are involved).
  time REAL NOT NULL,

  -- Virtual columns to index events on.
  code TEXT NOT NULL AS (json_extract(event, '$.code'))
);
CREATE INDEX events_change_time ON events_change(time, code);
