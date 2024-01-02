-- Objects are generally stored as JSON objects with virtual columns for indexes.
CREATE TABLE IF NOT EXISTS store_cluster_spec(
  -- Cluster Spec object as a JSON blob.
  cluster_spec TEXT NOT NULL,

  -- Manually managed normalised columns for indexes (where virtual columns can't be used).
  ns_id TEXT NOT NULL,
  cluster_id TEXT NOT NULL,

  -- Table constraints
  PRIMARY KEY(ns_id, cluster_id)
);

CREATE TABLE IF NOT EXISTS store_namespace(
  -- namespace object as a JSON blob.
  namespace TEXT NOT NULL,

  -- Manually managed normalised columns for indexes (where virtual columns can't be used).
  id TEXT PRIMARY KEY NOT NULL,

  -- Virtual columns to index events on.
  status TEXT NOT NULL AS (json_extract(namespace, '$.status'))
);

CREATE TABLE IF NOT EXISTS store_platform(
  -- Platform object as a JSON blob.
  platform TEXT NOT NULL,

  -- Manually managed normalised columns for indexes (where virtual columns can't be used).
  ns_id TEXT NOT NULL,
  name TEXT NOT NULL,

  -- Virtual columns form index and optimised queries.
  active BOOLEAN NOT NULL AS (json_extract(platform, '$.active')),

  -- Table constraints
  PRIMARY KEY(ns_id, name)
);
