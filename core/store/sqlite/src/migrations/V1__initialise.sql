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
  -- Identifier of the namespace.
  id TEXT PRIMARY KEY NOT NULL
);

CREATE TABLE IF NOT EXISTS store_platform(
  -- Platform object as a JSON blob.
  platform TEXT NOT NULL,

  -- Manually managed normalised columns for indexes (where virtual columns can't be used).
  ns_id TEXT NOT NULL,
  name TEXT NOT NULL,

  -- TODO: virtual columns for indexes.

  -- Table constraints
  PRIMARY KEY(ns_id, name)
);
