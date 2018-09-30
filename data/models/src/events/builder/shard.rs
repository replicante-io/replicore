use super::super::super::Shard;
use super::super::Event;
use super::super::EventBuilder;
use super::super::EventPayload;


/// Build `Event`s that belongs to the shard family.
pub struct ShardBuilder {
    builder: EventBuilder,
}

impl ShardBuilder {
    /// Create a new shard event builder.
    pub fn builder(builder: EventBuilder) -> ShardBuilder {
        ShardBuilder { builder }
    }

    /// Build an `EventPayload::ShardAllocationNew`.
    pub fn shard_allocation_new(self, shard: Shard) -> Event {
        let data = EventPayload::ShardAllocationNew(shard);
        self.builder.build(data)
    }
}


#[cfg(test)]
mod tests {
    use replicante_agent_models::Shard as WireShard;
    use replicante_agent_models::ShardRole;
    use super::Shard;
    use super::Event;
    use super::EventPayload;

    #[test]
    fn shard_new() {
        let shard = WireShard::new("shard", ShardRole::Primary, None, None);
        let shard = Shard::new("cluster", "test", shard);
        let event = Event::builder().shard().shard_allocation_new(shard.clone());
        let expected = EventPayload::ShardAllocationNew(shard);
        assert_eq!(event.payload, expected);
    }
}
