use serde::Serialize;

use replicante_models_core::agent::Shard;

/// Shard attributes to serialise a Shard reference from an indexed view.
#[derive(Serialize)]
pub struct ShardRef<'view> {
  pub node_id: &'view str,
  pub shard_id: &'view str,
}

impl<'view> From<&'view Shard> for ShardRef<'view> {
    fn from(shard: &'view Shard) -> ShardRef<'view> {
        ShardRef {
            node_id: &shard.node_id,
            shard_id: &shard.shard_id,
        }
    }
}
