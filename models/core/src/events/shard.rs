use serde::Deserialize;
use serde::Serialize;

use super::Event;
use super::EventBuilder;
use super::Payload;
use crate::agent::Shard;

/// Metadata attached to shard allocation changed events.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct AllocationChanged {
    pub after: Shard,
    pub before: Shard,
    pub cluster_id: String,
    pub node_id: String,
    pub shard_id: String,
}

/// Enumerates all possible shrd events emitted by the system.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(tag = "event", content = "payload")]
// TODO: use when possible #[non_exhaustive]
pub enum ShardEvent {
    /// A shard on a node has changed.
    #[serde(rename = "SHARD_ALLOCATION_CHANGED")]
    AllocationChanged(Box<AllocationChanged>),

    /// A shard was found for the first time on a node.
    #[serde(rename = "SHARD_ALLOCATION_NEW")]
    AllocationNew(Shard),
}

impl ShardEvent {
    /// Look up the cluster ID for the event, if they have one.
    pub fn cluster_id(&self) -> Option<&str> {
        let cluster_id = match self {
            ShardEvent::AllocationChanged(change) => &change.cluster_id,
            ShardEvent::AllocationNew(shard) => &shard.cluster_id,
        };
        Some(cluster_id)
    }

    /// Returns the event "code", the string that represents the event type.
    pub fn code(&self) -> &'static str {
        match self {
            ShardEvent::AllocationChanged(_) => "SHARD_ALLOCATION_CHANGED",
            ShardEvent::AllocationNew(_) => "SHARD_ALLOCATION_NEW",
        }
    }

    /// Returns the "ordering ID" for correctly streaming the event.
    pub fn stream_key(&self) -> &str {
        self.cluster_id().unwrap_or("<system>")
    }
}

/// Build `ClusterEvent`s, validating inputs.
pub struct ShardEventBuilder {
    pub(super) builder: EventBuilder,
}

impl ShardEventBuilder {
    /// Build a `ShardEvent::AllocationChanged` event.
    pub fn allocation_changed(self, before: Shard, after: Shard) -> Event {
        let cluster_id = before.cluster_id.clone();
        let node_id = before.node_id.clone();
        let shard_id = before.shard_id.clone();
        let event = ShardEvent::AllocationChanged(Box::new(AllocationChanged {
            after,
            before,
            cluster_id,
            node_id,
            shard_id,
        }));
        let payload = Payload::Shard(event);
        self.builder.finish(payload)
    }

    /// Build a `ShardEvent::AllocationNew` event.
    pub fn new_allocation(self, shard: Shard) -> Event {
        let event = ShardEvent::AllocationNew(shard);
        let payload = Payload::Shard(event);
        self.builder.finish(payload)
    }
}

#[cfg(test)]
mod tests {
    use super::AllocationChanged;
    use super::Event;
    use super::Payload;
    use super::ShardEvent;
    use crate::agent::Shard;
    use crate::agent::ShardRole;

    #[test]
    fn allocation_changed() {
        let after = Shard {
            cluster_id: "cluster".into(),
            commit_offset: None,
            lag: None,
            node_id: "node".into(),
            role: ShardRole::Secondary,
            shard_id: "shard".into(),
        };
        let before = Shard {
            cluster_id: "cluster".into(),
            commit_offset: None,
            lag: None,
            node_id: "node".into(),
            role: ShardRole::Primary,
            shard_id: "shard".into(),
        };
        let event = Event::builder()
            .shard()
            .allocation_changed(before.clone(), after.clone());
        let expected = Payload::Shard(ShardEvent::AllocationChanged(Box::new(AllocationChanged {
            after,
            cluster_id: before.cluster_id.clone(),
            node_id: before.node_id.clone(),
            shard_id: before.shard_id.clone(),
            before,
        })));
        assert_eq!(event.payload, expected);
    }

    #[test]
    fn new_allocation() {
        let shard = Shard {
            cluster_id: "cluster".into(),
            commit_offset: None,
            lag: None,
            node_id: "node".into(),
            role: ShardRole::Primary,
            shard_id: "shard".into(),
        };
        let event = Event::builder().shard().new_allocation(shard.clone());
        let expected = Payload::Shard(ShardEvent::AllocationNew(shard));
        assert_eq!(event.payload, expected);
    }
}
