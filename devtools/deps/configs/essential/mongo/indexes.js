/*** PRIMARY STORE ***/
db = db.getSiblingDB("replicore");

// Create indexes if missing.
//   Unique indexes for data integrity.
db.actions.createIndex({cluster_id: 1, action_id: 1}, {unique: true});
db.actions_orchestrator.createIndex({cluster_id: 1, action_id: 1}, {unique: true});
db.agents.createIndex({cluster_id: 1, host: 1}, {unique: true});
db.agents_info.createIndex({cluster_id: 1, host: 1}, {unique: true});
db.cluster_settings.createIndex({cluster_id: 1}, {unique: true});
db.clusters_meta.createIndex({cluster_id: 1}, {unique: true});
db.discoveries.createIndex({cluster_id: 1}, {unique: true});
db.discovery_settings.createIndex({namespace: 1, name: 1}, {unique: true});
db.nodes.createIndex({cluster_id: 1, node_id: 1}, {unique: true});
db.shards.createIndex({cluster_id: 1, shard_id: 1, node_id: 1}, {unique: true});

//   Indexes for performance reasons.
db.actions.createIndex({cluster_id: 1, node_id: 1, action_id: 1}, {unique: true});
db.clusters_meta.createIndex({shards: -1, nodes: -1, cluster_id: 1});
db.clusters_meta.createIndex({cluster_display_name: 1});
db.cluster_settings.createIndex({next_orchestrate: 1});
db.discovery_settings.createIndex({next_run: 1});

//   TTL indexes for cleanup (14 days).
db.actions.createIndex({finished_ts: 1}, {expireAfterSeconds: 1209600});
db.actions_orchestrator.createIndex({finished_ts: 1}, {expireAfterSeconds: 1209600});


/*** VIEW STORE ***/
db = db.getSiblingDB("repliview");

// Create indexes if missing.
//   Unique indexes for data integrity.
db.actions.createIndex({cluster_id: 1, action_id: 1}, {unique: true});
db.actions_history.createIndex({
  cluster_id: 1,
  action_id: 1,
  timestamp: 1,
  state: 1,
}, {unique: true});
db.actions_orchestrator.createIndex({cluster_id: 1, action_id: 1}, {unique: true});

//   Indexes for performance reasons.
db.actions.createIndex({cluster_id: 1, created_ts: -1});
db.actions_orchestrator.createIndex({cluster_id: 1, created_ts: -1});
db.cluster_orchestrate_report.createIndex({cluster_id: 1}, {unique: true});

//   TTL index lasting 14 days.
db.actions.createIndex({finished_ts: 1}, {expireAfterSeconds: 1209600});
db.actions_history.createIndex({finished_ts: 1}, {expireAfterSeconds: 1209600});
db.actions_orchestrator.createIndex({finished_ts: 1}, {expireAfterSeconds: 1209600});
db.events.createIndex({timestamp: 1}, {expireAfterSeconds: 1209600});
