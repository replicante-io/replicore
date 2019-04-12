use super::super::super::Shard;
use super::super::Event;
use super::super::EventBuilder;
use super::super::EventPayload;
use super::super::ShardAllocationChanged;

/// Build `Event`s that belongs to the shard family.
pub struct ShardBuilder {
    builder: EventBuilder,
}

impl ShardBuilder {
    /// Create a new shard event builder.
    pub fn builder(builder: EventBuilder) -> ShardBuilder {
        ShardBuilder { builder }
    }

    /// Build an `EventPayload::ShardAllocationChanged` event.
    pub fn allocation_changed(self, before: Shard, after: Shard) -> Event {
        let cluster_id = before.cluster_id.clone();
        let node_id = before.node_id.clone();
        let shard_id = before.shard_id.clone();
        let data = EventPayload::ShardAllocationChanged(ShardAllocationChanged {
            after,
            before,
            cluster_id,
            node_id,
            shard_id,
        });
        self.builder.build(data)
    }

    /// Build an `EventPayload::ShardAllocationNew` event.
    pub fn shard_allocation_new(self, shard: Shard) -> Event {
        let data = EventPayload::ShardAllocationNew(shard);
        self.builder.build(data)
    }
}

#[cfg(test)]
mod tests {
    use replicante_agent_models::Shard as WireShard;
    use replicante_agent_models::ShardRole;
    use super::Event;
    use super::EventPayload;
    use super::Shard;
    use super::ShardAllocationChanged;

    #[test]
    fn allocation_changed() {
        let before = WireShard::new("shard", ShardRole::Primary, None, None);
        let before = Shard::new("cluster", "test", before);
        let after = WireShard::new("shard", ShardRole::Secondary, None, None);
        let after = Shard::new("cluster", "test", after);
        let event = Event::builder().shard().allocation_changed(before.clone(), after.clone());
        let expected = EventPayload::ShardAllocationChanged(ShardAllocationChanged {
            after,
            cluster_id: before.cluster_id.clone(),
            node_id: before.node_id.clone(),
            shard_id: before.shard_id.clone(),
            before,
        });
        assert_eq!(event.payload, expected);
    }

    #[test]
    fn allocation_new() {
        let shard = WireShard::new("shard", ShardRole::Primary, None, None);
        let shard = Shard::new("cluster", "test", shard);
        let event = Event::builder().shard().shard_allocation_new(shard.clone());
        let expected = EventPayload::ShardAllocationNew(shard);
        assert_eq!(event.payload, expected);
    }
}
