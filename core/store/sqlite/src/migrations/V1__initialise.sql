-- Objects are generally stored as JSON objects with virtual columns for indexes.
CREATE TABLE IF NOT EXISTS store_cluster_converge_state(
  -- Cluster Converge State object as a JSON blob.
  cluster_state TEXT NOT NULL,

  -- Manually managed normalised columns for indexes (where virtual columns can't be used).
  ns_id TEXT NOT NULL,
  cluster_id TEXT NOT NULL,

  -- Table constraints
  PRIMARY KEY(ns_id, cluster_id)
);

CREATE TABLE IF NOT EXISTS store_cluster_disc(
  -- Cluster Discovery object as a JSON blob.
  cluster_disc TEXT NOT NULL,

  -- Manually managed normalised columns for indexes (where virtual columns can't be used).
  ns_id TEXT NOT NULL,
  cluster_id TEXT NOT NULL,

  -- Table constraints
  PRIMARY KEY(ns_id, cluster_id)
);

CREATE TABLE IF NOT EXISTS store_cluster_spec(
  -- Cluster Spec object as a JSON blob.
  cluster_spec TEXT NOT NULL,

  -- Manually managed normalised columns for indexes (where virtual columns can't be used).
  ns_id TEXT NOT NULL,
  cluster_id TEXT NOT NULL,

  -- Virtual columns form index and optimised queries.
  active BOOLEAN NOT NULL AS (json_extract(cluster_spec, '$.active')),

  -- Table constraints
  PRIMARY KEY(ns_id, cluster_id)
);

CREATE TABLE IF NOT EXISTS store_cluster_node(
  -- Node object as a JSON blob.
  node TEXT NOT NULL,

  -- Manually managed normalised columns for indexes (where virtual columns can't be used).
  ns_id TEXT NOT NULL,
  cluster_id TEXT NOT NULL,
  node_id TEXT NOT NULL,

  -- Table constraints
  PRIMARY KEY(ns_id, cluster_id, node_id)
);

CREATE TABLE IF NOT EXISTS store_namespace(
  -- namespace object as a JSON blob.
  namespace TEXT NOT NULL,

  -- Manually managed normalised columns for indexes (where virtual columns can't be used).
  id TEXT PRIMARY KEY NOT NULL,

  -- Virtual columns to index events on.
  status TEXT NOT NULL AS (json_extract(namespace, '$.status'))
);

CREATE TABLE IF NOT EXISTS store_oaction(
  -- Orchestrator action object as a JSON blob.
  oaction TEXT NOT NULL,

  -- Manually managed normalised columns for indexes (where virtual columns can't be used).
  ns_id TEXT NOT NULL,
  cluster_id TEXT NOT NULL,
  action_id TEXT NOT NULL,

  -- Times sorted and queried on use REAL for SQLite to operate on it correctly.
  created_ts REAL NOT NULL,
  finished_ts REAL DEFAULT NULL,

  -- Table constraints
  PRIMARY KEY(ns_id, cluster_id, action_id)
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

CREATE TABLE IF NOT EXISTS store_cluster_store_extras(
  -- StoreExtras object as a JSON blob.
  store_extras TEXT NOT NULL,

  -- Manually managed normalised columns for indexes (where virtual columns can't be used).
  ns_id TEXT NOT NULL,
  cluster_id TEXT NOT NULL,
  node_id TEXT NOT NULL,

  -- Table constraints
  PRIMARY KEY(ns_id, cluster_id, node_id)
);
