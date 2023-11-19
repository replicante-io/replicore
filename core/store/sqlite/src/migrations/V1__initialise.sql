-- Objects are generally stored as JSON objects with virtual columns for indexes.
CREATE TABLE IF NOT EXISTS store_namespace(
  -- Namespace object as a JSON blob.
  namespace TEXT NOT NULL,

  -- Manually managed normalised columns for indexes (where virtual columns can't be used).
  id TEXT PRIMARY KEY NOT NULL
);
