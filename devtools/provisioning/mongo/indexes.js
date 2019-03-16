// Create indexes if missing.
db = db.getSiblingDB("replicore");

/// Unique indexes for data integrity.
db.agents.createIndex({cluster: 1, host: 1}, {unique: true});
db.agents_info.createIndex({cluster: 1, host: 1}, {unique: true});
db.clusters_meta.createIndex({name: 1}, {unique: true});
db.discoveries.createIndex({cluster: 1}, {unique: true});
db.nodes.createIndex({cluster: 1, name: 1}, {unique: true});
db.shards.createIndex({cluster: 1, node: 1, id: 1}, {unique: true});

// TTL index lasting 14 days.
db.events.createIndex({timestamp: 1}, {expireAfterSeconds: 1209600});

// Indexes for performance reasons.
db.clusters_meta.createIndex({nodes: -1, name: 1});
