// Create indexes if missing.
db = db.getSiblingDB("replicore");

/// Unique indexes for data integrity.
db.agents.createIndex({cluster_id: 1, host: 1}, {unique: true});
db.agents_info.createIndex({cluster_id: 1, host: 1}, {unique: true});
db.clusters_meta.createIndex({cluster_id: 1}, {unique: true});
db.discoveries.createIndex({cluster_id: 1}, {unique: true});
db.nodes.createIndex({cluster_id: 1, node_id: 1}, {unique: true});
db.shards.createIndex({cluster_id: 1, node_id: 1, shard_id: 1}, {unique: true});

// TTL index lasting 14 days.
db.events.createIndex({timestamp: 1}, {expireAfterSeconds: 1209600});

// Indexes for performance reasons.
db.clusters_meta.createIndex({nodes: -1, cluster_id: 1});
